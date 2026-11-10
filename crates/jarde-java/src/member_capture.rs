//! Shared Java source capture certificates over same-run JVM facts and SSA.
//!
//! The caller supplies selected facts and performs requested method analysis. These certificates
//! never select an environment, open an artifact, publish source or grant artifact binding.
use crate::source_proof::{MemberCaptureProof, MemberCaptureRead};
use jarde_jvm::method_ir::{Definition, MethodIr, Slot, SsaTable, ValueId};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{
    ClassMemberFacts, CpEntryFacts, CpEntryKind, DescriptorKind, MethodCodeFacts, cp_entry,
    descriptor_facts,
};
use jarde_reader::error::Result;
use jarde_reader::model::ExecutionReport;
use jarde_reader::model::{JvmBytes, PhysicalDefinitionId, PhysicalMethodId};

/// This proof is deliberately narrower than the existing public `prove_target`: it uses the
/// already selected member relation and accepts a package-private physical constructor.
pub fn prove_family_capture(
    root: &[u8],
    child: &ClassMemberFacts,
    methods: &[(PhysicalMethodId, &MethodIr)],
    budget: &mut Budget,
) -> Result<std::result::Result<MemberCaptureProof, String>> {
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    if child.stopped_at.is_some()
        || child.fields.len() as u64 != child.field_count
        || child.methods.len() as u64 != child.method_count
    {
        return refuse("member physical tables are incomplete");
    }
    if child.methods.iter().any(|method| {
        !method
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Code")
    }) {
        return refuse("member method without bytecode has unknown capture uses");
    }
    let outer_descriptor = [b"L".as_slice(), root, b";"].concat();
    let candidates: Vec<_> = child
        .fields
        .iter()
        .enumerate()
        .filter(|(_, field)| {
            field.descriptor.raw().0 == outer_descriptor
                && field.access_flags & (0x1000 | 0x0010 | 0x0008) == 0x1010
        })
        .collect();
    let [(field_index, field)] = candidates.as_slice() else {
        return refuse("capture requires one synthetic final instance Outer field");
    };
    if child
        .fields
        .iter()
        .filter(|field| field.descriptor.raw().0 == outer_descriptor)
        .count()
        != 1
    {
        return refuse("another Outer-typed field prevents unique capture identity");
    }
    let constructors: Vec<_> = child
        .methods
        .iter()
        .filter(|method| method.name.raw().0 == b"<init>")
        .collect();
    let [constructor] = constructors.as_slice() else {
        return refuse(
            "capture requires one physical constructor; this() chains are outside this proof",
        );
    };
    let descriptor = &constructor.descriptor.raw().0;
    let Ok(parsed) = descriptor_facts(descriptor, DescriptorKind::Method) else {
        return refuse("constructor descriptor could not be parsed");
    };
    if parsed
        .parameters()
        .first()
        .and_then(|part| part.bytes(descriptor))
        != Some(outer_descriptor.as_slice())
    {
        return refuse("constructor first physical parameter is not Outer");
    }
    let Some((constructor_id, constructor_ir)) = methods
        .iter()
        .find(|(id, _)| id.name.0 == b"<init>" && id.descriptor.0 == *descriptor)
    else {
        return refuse("constructor SSA is unavailable");
    };
    let (Some(code), Some(ssa)) = (constructor_ir.code(), constructor_ir.ssa()) else {
        return refuse("constructor code or SSA is unavailable");
    };
    let instructions = &code.instructions;
    if !capture_constructor_shape(code) {
        return refuse(
            "constructor capture prologue or exception range is outside the proved shape",
        );
    }
    let pool = constructor_ir.constant_pool();
    for entry in pool {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if let CpEntryKind::MethodHandle {
            reference_index, ..
        } = entry.kind
        {
            if field_reference_matches(
                pool,
                Some(reference_index),
                &child.this_class.raw().0,
                &field.name.raw().0,
                &outer_descriptor,
            ) {
                return refuse("capture field has a method-handle use outside direct SSA reads");
            }
        }
    }
    if !field_reference_matches(
        pool,
        instructions[2].constant_pool_index,
        &child.this_class.raw().0,
        &field.name.raw().0,
        &outer_descriptor,
    ) {
        return refuse("constructor prologue writes a different field");
    }
    if !matches!(cp_entry(pool, instructions[4].constant_pool_index.unwrap_or(0)).ok().map(|entry| &entry.kind),
        Some(CpEntryKind::MethodRef { owner, name, descriptor, .. })
            if child.super_class.as_ref().is_some_and(|superclass| owner.0 == superclass.raw().0)
                && name.0 == b"<init>" && descriptor.0 == b"()V")
    {
        return refuse("constructor must invoke the direct zero-argument superclass constructor");
    }
    let Some(write) = ssa_instruction(ssa, 2) else {
        return refuse("capture write has no SSA instruction");
    };
    if write.reads().len() != 2
        || !write
            .reads()
            .iter()
            .any(|(_, value)| value_from_entry_load(ssa, *value, Slot::Local(0), 0))
        || !write
            .reads()
            .iter()
            .any(|(_, value)| value_from_entry_load(ssa, *value, Slot::Local(1), 1))
    {
        return refuse("capture write does not consume this and the first physical parameter");
    }
    let mut reads = Vec::new();
    for (method_id, ir) in methods {
        budget.poll()?;
        let (Some(code), Some(ssa)) = (ir.code(), ir.ssa()) else {
            return refuse("member method code or SSA is unavailable");
        };
        match scan_capture_method_uses(
            method_id,
            ir,
            code,
            ssa,
            constructor_id,
            &child.this_class.raw().0,
            &field.name.raw().0,
            &outer_descriptor,
            2,
            CaptureReadReceiver::EntryThis,
            budget,
        )? {
            Ok(method_reads) => reads.extend(method_reads),
            Err(reason) => return Ok(Err(reason)),
        }
    }
    let field_name = field.name.raw().clone();
    Ok(Ok(MemberCaptureProof {
        field_index: *field_index as u64,
        field_name: field_name.clone(),
        constructor: constructor_id.clone(),
        write_bci: 2,
        reads,
    }))
}

/// The general non-static member capture: the constructor may hold ordinary source work of its
/// own beside the synthetic outer write. Every identity fact the six-instruction certificate
/// states still holds — one synthetic final instance Outer field and no other Outer-typed field,
/// one constructor whose first physical parameter is the Outer, a method-handle-free field — and
/// two facts a body-holding constructor adds before its synthetic parameter can be elided: the
/// write reads exactly `this` and that parameter wherever the body states them, and that
/// parameter's value has no other consumer. A body with an exception handler keeps the
/// separated presentation: this fold removes a statement by its primary origin alone.
pub fn prove_family_instance_capture(
    root: &[u8],
    child: &ClassMemberFacts,
    methods: &[(PhysicalMethodId, &MethodIr)],
    budget: &mut Budget,
) -> Result<std::result::Result<MemberCaptureProof, String>> {
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    if child.stopped_at.is_some()
        || child.fields.len() as u64 != child.field_count
        || child.methods.len() as u64 != child.method_count
    {
        return refuse("member physical tables are incomplete");
    }
    if child.methods.iter().any(|method| {
        !method
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Code")
    }) {
        return refuse("member method without bytecode has unknown capture uses");
    }
    let outer_descriptor = [b"L".as_slice(), root, b";"].concat();
    let candidates: Vec<_> = child
        .fields
        .iter()
        .enumerate()
        .filter(|(_, field)| {
            field.descriptor.raw().0 == outer_descriptor
                && field.access_flags & (0x1000 | 0x0010 | 0x0008) == 0x1010
        })
        .collect();
    let [(field_index, field)] = candidates.as_slice() else {
        return refuse("capture requires one synthetic final instance Outer field");
    };
    if child
        .fields
        .iter()
        .filter(|field| field.descriptor.raw().0 == outer_descriptor)
        .count()
        != 1
    {
        return refuse("another Outer-typed field prevents unique capture identity");
    }
    let constructors: Vec<_> = child
        .methods
        .iter()
        .filter(|method| method.name.raw().0 == b"<init>")
        .collect();
    let [constructor] = constructors.as_slice() else {
        return refuse(
            "capture requires one physical constructor; this() chains are outside this proof",
        );
    };
    let descriptor = &constructor.descriptor.raw().0;
    let Ok(parsed) = descriptor_facts(descriptor, DescriptorKind::Method) else {
        return refuse("constructor descriptor could not be parsed");
    };
    if parsed
        .parameters()
        .first()
        .and_then(|part| part.bytes(descriptor))
        != Some(outer_descriptor.as_slice())
    {
        return refuse("constructor first physical parameter is not Outer");
    }
    let Some((constructor_id, constructor_ir)) = methods
        .iter()
        .find(|(id, _)| id.name.0 == b"<init>" && id.descriptor.0 == *descriptor)
    else {
        return refuse("constructor SSA is unavailable");
    };
    let (Some(code), Some(ssa)) = (constructor_ir.code(), constructor_ir.ssa()) else {
        return refuse("constructor code or SSA is unavailable");
    };
    if code.stopped_at.is_some() || !code.exception_handlers.is_empty() {
        return refuse("constructor body has an exception handler or stopped early");
    }
    let pool = constructor_ir.constant_pool();
    for entry in pool {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if let CpEntryKind::MethodHandle {
            reference_index, ..
        } = entry.kind
        {
            if field_reference_matches(
                pool,
                Some(reference_index),
                &child.this_class.raw().0,
                &field.name.raw().0,
                &outer_descriptor,
            ) {
                return refuse("capture field has a method-handle use outside direct SSA reads");
            }
        }
    }
    // The synthetic outer write: the one putfield of the capture field whose SSA reads are the
    // entry `this` and the entry first parameter, wherever the body states them.
    let mut writes = Vec::new();
    for instruction in &code.instructions {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if instruction.opcode != 0xb5 {
            continue;
        }
        if !field_reference_matches(
            pool,
            instruction.constant_pool_index,
            &child.this_class.raw().0,
            &field.name.raw().0,
            &outer_descriptor,
        ) {
            continue;
        }
        let Some(write) = ssa_instruction(ssa, instruction.bci) else {
            return refuse("capture write has no SSA instruction");
        };
        if write.reads().len() != 2
            || !write
                .reads()
                .iter()
                .any(|(_, value)| value_from_this_load(ssa, *value))
            || !write
                .reads()
                .iter()
                .any(|(_, value)| value_from_entry_slot_load(ssa, *value, Slot::Local(1)))
        {
            return refuse("capture write does not consume this and the first physical parameter");
        }
        writes.push(instruction.bci);
    }
    let [write_bci] = writes.as_slice() else {
        return refuse("constructor must write the capture field exactly once");
    };
    let write_bci = *write_bci;
    // The dropped parameter keeps no other reference: its entry value is loaded exactly once,
    // for this write.
    let Some(write) = ssa_instruction(ssa, write_bci) else {
        return refuse("capture write has no SSA instruction");
    };
    let Some((_, parameter_load)) = write
        .reads()
        .iter()
        .find(|(_, value)| value_from_entry_slot_load(ssa, *value, Slot::Local(1)))
    else {
        return refuse("capture write does not consume the first physical parameter");
    };
    let Definition::Instruction {
        bci: parameter_load_bci,
        ..
    } = ssa.value(*parameter_load).def()
    else {
        return refuse("first physical parameter load is not an instruction");
    };
    let parameter_load_bci = *parameter_load_bci;
    let Some(parameter_load) = ssa_instruction(ssa, parameter_load_bci) else {
        return refuse("first physical parameter load has no SSA instruction");
    };
    let Some((_, parameter_entry)) = parameter_load
        .reads()
        .iter()
        .find(|(slot, _)| *slot == Slot::Local(1))
    else {
        return refuse("first physical parameter load reads no entry value");
    };
    let parameter_uses = ssa.value(*parameter_entry).uses();
    if parameter_uses.len() != 1 || parameter_uses[0].bci() != Some(parameter_load_bci) {
        return refuse("first physical parameter has a consumer besides the capture write");
    }
    let mut reads = Vec::new();
    for (method_id, ir) in methods {
        budget.poll()?;
        let (Some(code), Some(ssa)) = (ir.code(), ir.ssa()) else {
            return refuse("member method code or SSA is unavailable");
        };
        match scan_capture_method_uses(
            method_id,
            ir,
            code,
            ssa,
            constructor_id,
            &child.this_class.raw().0,
            &field.name.raw().0,
            &outer_descriptor,
            write_bci,
            CaptureReadReceiver::EntryThis,
            budget,
        )? {
            Ok(method_reads) => reads.extend(method_reads),
            Err(reason) => return Ok(Err(reason)),
        }
    }
    let field_name = field.name.raw().clone();
    Ok(Ok(MemberCaptureProof {
        field_index: *field_index as u64,
        field_name: field_name.clone(),
        constructor: constructor_id.clone(),
        write_bci,
        reads,
    }))
}

/// The value one specific entry slot holds, loaded anywhere in this body.
fn value_from_entry_slot_load(ssa: &SsaTable, value: ValueId, slot: Slot) -> bool {
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return false;
    };
    value_from_entry_load(ssa, value, slot, *bci)
}

/// Proves the one javac Java 8 local-`double` capture shape used by anonymous interfaces. The
/// caller separately closes the root allocation argument and whole-input owner census; this
/// certificate covers only the child's physical field, constructor, SSA reads, and field uses.
pub fn prove_anonymous_double_capture(
    child: &ClassMemberFacts,
    methods: &[(PhysicalMethodId, &MethodIr)],
    budget: &mut Budget,
) -> Result<std::result::Result<MemberCaptureProof, String>> {
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    if child.stopped_at.is_some()
        || child.fields.len() as u64 != child.field_count
        || child.methods.len() as u64 != child.method_count
        || child.field_count != 1
    {
        return refuse("anonymous double capture requires complete tables with one field");
    }
    if child.methods.iter().any(|method| {
        !method
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Code")
    }) {
        return refuse("anonymous method without bytecode has unknown capture uses");
    }
    let [field] = child.fields.as_slice() else {
        return refuse("anonymous double capture requires one physical field");
    };
    if field.descriptor.raw().0 != b"D" || field.access_flags & (0x1000 | 0x0010 | 0x0008) != 0x1010
    {
        return refuse("capture requires one synthetic final instance double field");
    }
    let constructors: Vec<_> = child
        .methods
        .iter()
        .filter(|method| method.name.raw().0 == b"<init>")
        .collect();
    let [constructor] = constructors.as_slice() else {
        return refuse("capture requires one physical constructor");
    };
    if constructor.descriptor.raw().0 != b"(D)V" {
        return refuse("capture constructor is not exactly (D)V");
    }
    let Some((constructor_id, constructor_ir)) = methods
        .iter()
        .find(|(id, _)| id.name.0 == b"<init>" && id.descriptor.0 == b"(D)V")
    else {
        return refuse("constructor SSA is unavailable");
    };
    let (Some(code), Some(ssa)) = (constructor_ir.code(), constructor_ir.ssa()) else {
        return refuse("constructor code or SSA is unavailable");
    };
    if !anonymous_double_constructor_shape(code) {
        return refuse(
            "constructor capture prologue or exception range is outside the proved shape",
        );
    }
    let owner = &child.this_class.raw().0;
    let name = &field.name.raw().0;
    let pool = constructor_ir.constant_pool();
    for entry in pool {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if let CpEntryKind::MethodHandle {
            reference_index, ..
        } = entry.kind
            && field_reference_matches(pool, Some(reference_index), owner, name, b"D")
        {
            return refuse("capture field has a method-handle use outside direct SSA reads");
        }
    }
    if !field_reference_matches(
        pool,
        code.instructions[2].constant_pool_index,
        owner,
        name,
        b"D",
    ) || !matches!(cp_entry(pool, code.instructions[4].constant_pool_index.unwrap_or(0)).ok().map(|entry| &entry.kind),
            Some(CpEntryKind::MethodRef { owner, name, descriptor, .. })
                if child.super_class.as_ref().is_some_and(|superclass| owner.0 == superclass.raw().0)
                    && name.0 == b"<init>" && descriptor.0 == b"()V")
    {
        return refuse(
            "constructor prologue writes a different field or calls a different superclass",
        );
    }
    let Some(write) = ssa_instruction(ssa, 2) else {
        return refuse("capture write has no SSA instruction");
    };
    if write.reads().len() != 2
        || !write
            .reads()
            .iter()
            .any(|(_, value)| value_from_this_load(ssa, *value))
        || !write
            .reads()
            .iter()
            .any(|(_, value)| value_from_entry_load(ssa, *value, Slot::Local(1), 1))
    {
        return refuse("capture write does not consume this and the double entry parameter");
    }
    let mut reads = Vec::new();
    for (method_id, ir) in methods {
        budget.poll()?;
        let (Some(code), Some(ssa)) = (ir.code(), ir.ssa()) else {
            return refuse("anonymous method code or SSA is unavailable");
        };
        match scan_capture_method_uses(
            method_id,
            ir,
            code,
            ssa,
            constructor_id,
            owner,
            name,
            b"D",
            2,
            CaptureReadReceiver::EntryThis,
            budget,
        )? {
            Ok(method_reads) => reads.extend(method_reads),
            Err(reason) => return Ok(Err(reason)),
        }
    }
    if reads.is_empty() {
        return refuse("capture field has no proved child read");
    }
    let field_name = JvmBytes(name.to_vec());
    Ok(Ok(MemberCaptureProof {
        field_index: 0,
        field_name: field_name.clone(),
        constructor: constructor_id.clone(),
        write_bci: 2,
        reads,
    }))
}

/// The parameter roles one anonymous `val$`-capture constructor proves. The partition is the
/// whole constructor shape: every physical parameter is consumed by exactly one proved sink, and
/// everything else the body holds is one of the two sinks, a parameter load or the closing
/// return.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnonymousValConstructorPartition {
    /// The one `invokespecial <init>` on the direct superclass.
    pub super_invoke_bci: u32,
    /// The descriptor that invoke names: the child constructor descriptor with the capture
    /// parameters removed.
    pub super_descriptor: Vec<u8>,
    /// Physical parameter slots feeding the super invoke, in argument order.
    pub super_parameter_slots: Vec<u16>,
    /// The one `putfield` writing the capture field.
    pub capture_write_bci: u32,
    /// Physical parameter slots stored into the capture field.
    pub capture_parameter_slots: Vec<u16>,
}

/// Splits one anonymous `val$`-capture constructor's physical parameters into the superclass
/// arguments and the capture stores by their SSA consumption points alone.
///
/// javac mints the constructor with one load per physical parameter and exactly two sinks: the
/// `putfield` of the synthetic capture field and the `invokespecial` of the direct superclass
/// constructor. The partition refuses any parameter that no proved sink consumes, any parameter
/// more than one sink or position consumes, any unproved sink at all, and any result whose
/// superclass arguments are not the leading physical parameters in order — javac keeps the source
/// order, so an argument that moved is not a constructor this projection can rebuild.
pub fn partition_anonymous_val_constructor(
    child: &ClassMemberFacts,
    field_name: &[u8],
    field_descriptor: &[u8],
    ir: &MethodIr,
    constructor_descriptor: &[u8],
    budget: &mut Budget,
) -> Result<std::result::Result<AnonymousValConstructorPartition, String>> {
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    let (Some(code), Some(ssa)) = (ir.code(), ir.ssa()) else {
        return refuse("constructor code or SSA is unavailable");
    };
    if code.stopped_at.is_some()
        || code.exception_handler_count != 0
        || !code.exception_handlers.is_empty()
    {
        return refuse(
            "constructor capture prologue or exception range is outside the proved shape",
        );
    }
    let Ok(parsed) = descriptor_facts(constructor_descriptor, DescriptorKind::Method) else {
        return refuse("constructor descriptor could not be parsed");
    };
    let mut parameter_slots = Vec::new();
    let mut parameter_types = Vec::new();
    let mut next_slot = 1u16;
    for part in parsed.parameters() {
        let Some(bytes) = part.bytes(constructor_descriptor) else {
            return refuse("constructor descriptor parameter could not be read");
        };
        parameter_slots.push(next_slot);
        parameter_types.push(bytes.to_vec());
        // long and double occupy two local slots
        next_slot += 1 + u16::from(matches!(bytes.first(), Some(b'J' | b'D')));
    }
    let pool = ir.constant_pool();
    let owner = &child.this_class.raw().0;
    let mut writes = Vec::new();
    let mut invokes = Vec::new();
    for instruction in &code.instructions {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        match instruction.opcode {
            0xb5 => {
                if field_reference_matches(
                    pool,
                    instruction.constant_pool_index,
                    owner,
                    field_name,
                    field_descriptor,
                ) {
                    writes.push(instruction.bci);
                } else {
                    return refuse("constructor writes a field outside the proved capture");
                }
            }
            0xb7 => {
                let Some(CpEntryKind::MethodRef {
                    owner: target,
                    name,
                    descriptor,
                    ..
                }) = cp_entry(pool, instruction.constant_pool_index.unwrap_or(0))
                    .ok()
                    .map(|entry| &entry.kind)
                else {
                    return refuse("constructor invokes an unproved special target");
                };
                if !child
                    .super_class
                    .as_ref()
                    .is_some_and(|superclass| target.0 == superclass.raw().0)
                    || name.0 != b"<init>"
                {
                    return refuse("constructor invokes an unproved special target");
                }
                invokes.push((instruction.bci, descriptor.0.to_vec()));
            }
            _ => {}
        }
    }
    let [capture_write_bci] = writes.as_slice() else {
        return refuse("capture field must have exactly one constructor write");
    };
    let [(super_invoke_bci, super_descriptor)] = invokes.as_slice() else {
        return refuse("constructor must invoke the direct superclass constructor exactly once");
    };
    if !super_descriptor.ends_with(b")V") {
        return refuse("superclass constructor does not return void");
    }
    // One load per physical parameter: javac mints the constructor with exactly one consumption
    // point per parameter, so a second load is already a shape this proof does not rebuild.
    let mut loads: Vec<(u16, u32, ValueId)> = Vec::new();
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            let [(Slot::Local(read_slot), value)] = instruction.reads() else {
                continue;
            };
            let slot = *read_slot;
            if slot != 0 && !parameter_slots.contains(&slot) {
                continue;
            }
            if !(0x15..=0x2d).contains(&instruction.opcode()) {
                continue;
            }
            if !matches!(
                ssa.value(*value).def(),
                Definition::Entry {
                    slot: Slot::Local(entry),
                    ..
                } if *entry == slot
            ) {
                continue;
            }
            let [(_, output)] = instruction.writes() else {
                continue;
            };
            loads.push((slot, instruction.bci(), *output));
        }
    }
    let mut this_loads = Vec::new();
    let mut parameter_loads = Vec::new();
    for load in &loads {
        if load.0 == 0 {
            this_loads.push(load);
        } else {
            parameter_loads.push(load);
        }
    }
    let mut per_slot: std::collections::BTreeMap<u16, Vec<&(u16, u32, ValueId)>> =
        std::collections::BTreeMap::new();
    for load in &parameter_loads {
        per_slot.entry(load.0).or_default().push(load);
    }
    for slot in &parameter_slots {
        if per_slot.get(slot).is_none_or(|loads| loads.len() != 1) {
            return refuse("physical parameter is not loaded exactly once");
        }
    }
    for (_, _, output) in &this_loads {
        if ssa.value(*output).uses().len() != 1 {
            return refuse("receiver load flows into an unproved consumer");
        }
    }
    // Each parameter load's one consumer must be one of the two sinks, at the position the role
    // states: the capture write's value slot, or a superclass-argument slot.
    enum ParameterRole {
        Capture,
        SuperArgument(u32),
    }
    let mut roles: std::collections::BTreeMap<u16, ParameterRole> =
        std::collections::BTreeMap::new();
    for (slot, _, output) in &parameter_loads {
        let uses = ssa.value(*output).uses();
        let [use_] = uses else {
            return refuse("parameter load flows into an unproved consumer");
        };
        let Some(use_bci) = use_.bci() else {
            return refuse("parameter load flows through an unproved phi");
        };
        if use_bci == *capture_write_bci {
            let Some(write) = ssa_instruction(ssa, use_bci) else {
                return refuse("capture write has no SSA instruction");
            };
            let Some(read_index) = write.reads().iter().position(|(_, value)| value == output)
            else {
                return refuse("capture write does not consume the parameter value");
            };
            // the read order is the stack pop order, so the parameter may sit at either record;
            // what must hold is that the other read is the receiver
            if write.reads().len() != 2
                || !value_from_this_load(ssa, write.reads()[1 - read_index].1)
            {
                return refuse("capture write consumes the parameter at an unproved position");
            }
            if roles.insert(*slot, ParameterRole::Capture).is_some() {
                return refuse("physical parameter is consumed by two roles");
            }
            continue;
        }
        if use_bci == *super_invoke_bci {
            let Some(invoke) = ssa_instruction(ssa, use_bci) else {
                return refuse("superclass constructor invoke has no SSA instruction");
            };
            let Some(read_index) = invoke.reads().iter().position(|(_, value)| value == output)
            else {
                return refuse("superclass constructor does not consume the parameter value");
            };
            if read_index + 1 == invoke.reads().len() {
                return refuse("parameter load reaches the superclass receiver position");
            }
            // the recorded invoke reads end with the receiver, so the argument positions count
            // backwards from it
            let argument_index = (invoke.reads().len() - 2 - read_index) as u32;
            if roles
                .insert(*slot, ParameterRole::SuperArgument(argument_index))
                .is_some()
            {
                return refuse("physical parameter is consumed by two roles");
            }
            continue;
        }
        return refuse("parameter is consumed outside the proved capture and super roles");
    }
    let mut super_slots: Vec<(u16, u32)> = Vec::new();
    let mut capture_slots: Vec<u16> = Vec::new();
    for slot in &parameter_slots {
        match roles.get(slot) {
            Some(ParameterRole::Capture) => capture_slots.push(*slot),
            Some(ParameterRole::SuperArgument(index)) => super_slots.push((*slot, *index)),
            None => return refuse("physical parameter has no proved consumption"),
        }
    }
    // javac forwards the superclass arguments in source order, so they are the leading physical
    // parameters with counting argument positions, and the capture parameters are the rest.
    super_slots.sort_by_key(|(_, index)| *index);
    if super_slots
        .iter()
        .enumerate()
        .any(|(index, (slot, argument))| {
            *argument != index as u32 || parameter_slots[index] != *slot
        })
    {
        return refuse("superclass arguments are not the leading physical parameters in order");
    }
    if capture_slots != parameter_slots[super_slots.len()..] {
        return refuse("capture parameters are not the trailing physical parameters");
    }
    let Ok(super_parsed) = descriptor_facts(super_descriptor, DescriptorKind::Method) else {
        return refuse("superclass constructor descriptor could not be parsed");
    };
    let Some(super_parameters) = super_parsed
        .parameters()
        .iter()
        .map(|part| part.bytes(super_descriptor))
        .collect::<Option<Vec<_>>>()
    else {
        return refuse("superclass constructor descriptor could not be read");
    };
    if super_parameters.len() != super_slots.len()
        || super_parameters
            .iter()
            .zip(parameter_types.iter())
            .any(|(super_type, child_type)| *super_type != child_type.as_slice())
    {
        return refuse("superclass constructor descriptor does not match the forwarded parameters");
    }
    Ok(Ok(AnonymousValConstructorPartition {
        super_invoke_bci: *super_invoke_bci,
        super_descriptor: super_descriptor.clone(),
        super_parameter_slots: super_slots.into_iter().map(|(slot, _)| slot).collect(),
        capture_write_bci: *capture_write_bci,
        capture_parameter_slots: capture_slots,
    }))
}

/// Proves the one javac Java 8 local-capture shape that carries real superclass arguments: a
/// single synthetic final instance `val$` field, one constructor whose parameters split into the
/// proved super-argument prefix and the capture store, and every field read closed to its SSA
/// consumer. The caller separately closes the allocation site, the owner census and the
/// root-side argument value; this certificate covers only the child's physical facts.
pub fn prove_anonymous_val_capture(
    child: &ClassMemberFacts,
    methods: &[(PhysicalMethodId, &MethodIr)],
    receiver: CaptureReadReceiver,
    budget: &mut Budget,
) -> Result<std::result::Result<MemberCaptureProof, String>> {
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    if child.stopped_at.is_some()
        || child.fields.len() as u64 != child.field_count
        || child.methods.len() as u64 != child.method_count
        || child.field_count != 1
    {
        return refuse("anonymous val capture requires complete tables with one field");
    }
    if child.methods.iter().any(|method| {
        !method
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Code")
    }) {
        return refuse("anonymous method without bytecode has unknown capture uses");
    }
    let [field] = child.fields.as_slice() else {
        return refuse("anonymous val capture requires one physical field");
    };
    if field.access_flags & (0x1000 | 0x0010 | 0x0008) != 0x1010 {
        return refuse("capture requires one synthetic final instance field");
    }
    if !field.name.raw().0.starts_with(b"val$") {
        return refuse("capture field is not a javac local-capture name");
    }
    let field_descriptor = field.descriptor.raw().0.as_slice();
    let constructors: Vec<_> = child
        .methods
        .iter()
        .filter(|method| method.name.raw().0 == b"<init>")
        .collect();
    let [constructor] = constructors.as_slice() else {
        return refuse("capture requires one physical constructor");
    };
    let constructor_descriptor = constructor.descriptor.raw().0.clone();
    let Some((constructor_id, constructor_ir)) = methods
        .iter()
        .find(|(id, _)| id.name.0 == b"<init>" && id.descriptor.0 == constructor_descriptor)
    else {
        return refuse("constructor SSA is unavailable");
    };
    let partition = match partition_anonymous_val_constructor(
        child,
        &field.name.raw().0,
        field_descriptor,
        constructor_ir,
        &constructor_descriptor,
        budget,
    )? {
        Ok(partition) => partition,
        Err(reason) => return Ok(Err(reason)),
    };
    let [capture_slot] = partition.capture_parameter_slots.as_slice() else {
        return refuse("capture field must be stored from exactly one parameter");
    };
    let owner = &child.this_class.raw().0;
    let name = &field.name.raw().0;
    let pool = constructor_ir.constant_pool();
    for entry in pool {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if let CpEntryKind::MethodHandle {
            reference_index, ..
        } = entry.kind
            && field_reference_matches(pool, Some(reference_index), owner, name, field_descriptor)
        {
            return refuse("capture field has a method-handle use outside direct SSA reads");
        }
    }
    let Some(ssa) = constructor_ir.ssa() else {
        return refuse("constructor code or SSA is unavailable");
    };
    let Some(write) = ssa_instruction(ssa, partition.capture_write_bci) else {
        return refuse("capture write has no SSA instruction");
    };
    if write.reads().len() != 2
        || !write
            .reads()
            .iter()
            .any(|(_, value)| value_from_this_load(ssa, *value))
        || !write
            .reads()
            .iter()
            .any(|(_, value)| value_from_entry_slot_load(ssa, *value, Slot::Local(*capture_slot)))
    {
        return refuse("capture write does not consume this and the capture entry parameter");
    }
    let mut reads = Vec::new();
    for (method_id, ir) in methods {
        budget.poll()?;
        let (Some(code), Some(ssa)) = (ir.code(), ir.ssa()) else {
            return refuse("anonymous method code or SSA is unavailable");
        };
        match scan_capture_method_uses(
            method_id,
            ir,
            code,
            ssa,
            constructor_id,
            owner,
            name,
            field_descriptor,
            partition.capture_write_bci,
            receiver,
            budget,
        )? {
            Ok(method_reads) => reads.extend(method_reads),
            Err(reason) => return Ok(Err(reason)),
        }
    }
    if reads.is_empty() {
        return refuse("capture field has no proved child read");
    }
    let field_name = JvmBytes(name.to_vec());
    Ok(Ok(MemberCaptureProof {
        field_index: 0,
        field_name: field_name.clone(),
        constructor: constructor_id.clone(),
        write_bci: partition.capture_write_bci,
        reads,
    }))
}

/// The assignment facts of one root-method local slot a captured anonymous value may come from:
/// exactly one write (an effectively final local, with the write's BCI), or no write at all (an
/// unassigned parameter slot). A phi merge or a second assignment means the name does not denote
/// one value, and the capture is refused.
pub fn local_slot_single_write(
    ssa: &SsaTable,
    slot: u16,
) -> std::result::Result<Option<u32>, String> {
    let mut writes = Vec::new();
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            if instruction
                .writes()
                .iter()
                .any(|(write_slot, _)| *write_slot == Slot::Local(slot))
            {
                writes.push(instruction.bci());
            }
        }
    }
    if ssa.phis().iter().any(|phi| match phi.slot() {
        Slot::Local(phi_slot) => phi_slot == slot,
        Slot::Stack(_) => false,
    }) {
        return Err("captured local is merged by a phi".to_owned());
    }
    match writes.as_slice() {
        [bci] => Ok(Some(*bci)),
        [] => Ok(None),
        _ => Err("captured local has more than one assignment".to_owned()),
    }
}

#[allow(clippy::too_many_arguments)]
#[doc(hidden)]
pub fn scan_capture_method_uses(
    method: &PhysicalMethodId,
    ir: &MethodIr,
    code: &MethodCodeFacts,
    ssa: &SsaTable,
    constructor: &PhysicalMethodId,
    owner: &[u8],
    name: &[u8],
    descriptor: &[u8],
    write_bci: u32,
    receiver: CaptureReadReceiver,
    budget: &mut Budget,
) -> Result<std::result::Result<Vec<MemberCaptureRead>, String>> {
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    if code.stopped_at.is_some() {
        return refuse("member method code is incomplete");
    }
    let mut reads = Vec::new();
    for instruction in &code.instructions {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if !field_reference_matches(
            ir.constant_pool(),
            instruction.constant_pool_index,
            owner,
            name,
            descriptor,
        ) {
            continue;
        }
        if !matches!(instruction.opcode, 0xb4 | 0xb5) {
            return refuse("capture field has a non-instance or unknown bytecode use");
        }
        if instruction.opcode == 0xb5 {
            if !capture_write_is_unique_site(method, instruction.bci, constructor, write_bci) {
                return refuse("capture field has an extra write");
            }
            continue;
        }
        let Some(access) = ssa_instruction(ssa, instruction.bci) else {
            return refuse("capture read has no SSA instruction");
        };
        if access.reads().len() != 1 || !receiver.admits(ssa, access.reads()[0].1) {
            return refuse("capture read receiver is not the member this");
        }
        let [(_, result)] = access.writes() else {
            return refuse("capture read result has no unique SSA value");
        };
        if !matches!(ssa.value(*result).def(), Definition::Instruction { bci, .. } if *bci == instruction.bci)
        {
            return refuse("capture read result has another SSA definition");
        }
        let mut consumer_bcis = Vec::new();
        for use_ in ssa.value(*result).uses() {
            let Some(bci) = use_.bci() else {
                return refuse("capture read flows through an unproved phi");
            };
            if !ssa_instruction(ssa, bci)
                .is_some_and(|consumer| consumer.reads().iter().any(|(_, value)| value == result))
            {
                return refuse("capture read has an unknown SSA consumer");
            }
            consumer_bcis.push(bci);
        }
        reads.push(MemberCaptureRead {
            method: method.clone(),
            bci: instruction.bci,
            consumer_bcis,
        });
    }
    Ok(Ok(reads))
}

#[doc(hidden)]
pub fn capture_write_is_unique_site(
    method: &PhysicalMethodId,
    bci: u32,
    constructor: &PhysicalMethodId,
    write_bci: u32,
) -> bool {
    method == constructor && bci == write_bci
}

fn anonymous_double_constructor_shape(code: &MethodCodeFacts) -> bool {
    let instructions = &code.instructions;
    code.stopped_at.is_none()
        && code.exception_handler_count == 0
        && instructions.len() == 6
        && instructions[0].bci == 0
        && instructions[0].opcode == 0x2a
        && instructions[1].bci == 1
        && instructions[1].opcode == 0x27
        && instructions[2].bci == 2
        && instructions[2].opcode == 0xb5
        && instructions[3].bci == 5
        && instructions[3].opcode == 0x2a
        && instructions[4].bci == 6
        && instructions[4].opcode == 0xb7
        && instructions[5].bci == 9
        && instructions[5].opcode == 0xb1
        && code.exception_handlers.is_empty()
}

#[doc(hidden)]
pub fn capture_constructor_shape(code: &MethodCodeFacts) -> bool {
    let instructions = &code.instructions;
    code.stopped_at.is_none()
        && instructions.len() == 6
        && instructions[0].bci == 0
        && instructions[0].opcode == 0x2a
        && instructions[1].bci == 1
        && instructions[1].opcode == 0x2b
        && instructions[2].bci == 2
        && instructions[2].opcode == 0xb5
        && instructions[3].bci == 5
        && instructions[3].opcode == 0x2a
        && instructions[4].bci == 6
        && instructions[4].opcode == 0xb7
        && instructions[5].bci == 9
        && instructions[5].opcode == 0xb1
        && code.exception_handlers.is_empty()
}

fn field_reference_matches(
    pool: &[CpEntryFacts],
    index: Option<u16>,
    owner: &[u8],
    name: &[u8],
    descriptor: &[u8],
) -> bool {
    matches!(index.and_then(|index| cp_entry(pool, index).ok()).map(|entry| &entry.kind),
        Some(CpEntryKind::FieldRef { owner: actual_owner, name: actual_name, descriptor: actual_descriptor, .. })
            if actual_owner.0 == owner && actual_name.0 == name && actual_descriptor.0 == descriptor)
}

#[doc(hidden)]
pub fn ssa_instruction(ssa: &SsaTable, bci: u32) -> Option<&jarde_jvm::method_ir::SsaInstruction> {
    ssa.blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| instruction.bci() == bci)
}

#[doc(hidden)]
pub fn value_from_entry_load(ssa: &SsaTable, value: ValueId, slot: Slot, load_bci: u32) -> bool {
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return false;
    };
    if *bci != load_bci {
        return false;
    }
    let Some(load) = ssa_instruction(ssa, *bci) else {
        return false;
    };
    load.reads().iter().any(|(read_slot, source)| *read_slot == slot
        && matches!(ssa.value(*source).def(), Definition::Entry { slot: entry_slot, .. } if *entry_slot == slot))
}

#[doc(hidden)]
pub fn value_from_this_load(ssa: &SsaTable, value: ValueId) -> bool {
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return false;
    };
    value_from_entry_load(ssa, value, Slot::Local(0), *bci)
        && ssa_instruction(ssa, *bci).is_some_and(|load| load.opcode() == 0x2a)
}

/// Which `this` a capture-field read's receiver may be, as the two readings the certificates
/// state.
///
/// This is the containment pattern the anonymous family keeps for every widening: a certificate
/// admits one new shape only for the path whose slice proved it, so a widening on one slice can
/// never open another slice's acceptance set as a side effect.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CaptureReadReceiver {
    /// The entry `this` of an ordinary method body ([`value_from_this_load`]): the reading every
    /// companion-body projection's certificate states, because the bodies they present are the
    /// companion's *methods*.
    EntryThis,
    /// The same, or the **constructor's own `this` after its constructor call** — the value the
    /// call wrote into slot 0 (JVMS 4.9.2's conversion, which this layer states as that write).
    /// The double-brace allocation point (change `recover-double-brace-allocation-site`) reads a
    /// companion's whole body as its constructor, so every capture read it presents sits after
    /// that call, and the instance the read runs on is the same one the source's anonymous class
    /// would have.
    ConstructorThis,
}

impl CaptureReadReceiver {
    /// Whether one value is the `this` this reading admits as a field read's receiver.
    fn admits(self, ssa: &SsaTable, value: ValueId) -> bool {
        value_from_this_load(ssa, value)
            || (self == Self::ConstructorThis && value_from_constructor_this(ssa, value))
    }
}

/// Whether one value is the initialized `this` a constructor's own constructor call produced,
/// read either directly or through the `aload_0` that loads it.
///
/// The receiver of such a read is the same instance an entry `this` names — the constructor's own
/// — and the value that carries it is the JVMS 4.9.2 conversion and nothing else: an
/// `invokespecial` is the only instruction the frame pass lets convert the token, and this layer
/// records the conversion as that call's write of slot 0.
fn value_from_constructor_this(ssa: &SsaTable, value: ValueId) -> bool {
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return false;
    };
    constructor_this_write(ssa, value, *bci)
        || ssa_instruction(ssa, *bci).is_some_and(|load| {
            load.opcode() == 0x2a
                && load.reads().iter().any(|(slot, source)| {
                    *slot == Slot::Local(0)
                        && matches!(
                            ssa.value(*source).def(),
                            Definition::Instruction { bci, .. }
                                if constructor_this_write(ssa, *source, *bci)
                        )
                })
        })
}

/// Whether the instruction at `bci` is the constructor call that produced `value`: an
/// `invokespecial` writing slot 0 with it, whose own receiver is the entry `this` it converted.
fn constructor_this_write(ssa: &SsaTable, value: ValueId, bci: u32) -> bool {
    let Some(call) = ssa_instruction(ssa, bci) else {
        return false;
    };
    call.opcode() == 0xb7
        && call
            .writes()
            .iter()
            .any(|(slot, written)| *slot == Slot::Local(0) && *written == value)
        && call
            .reads()
            .iter()
            .any(|(_, source)| value_from_this_load(ssa, *source))
}

/// Prove an ordinary member capture from its ordered physical method demand.
/// The supplier owns actual IR execution and the original request-stop channel.
#[allow(clippy::too_many_arguments)]
pub fn prove_member_capture<F: From<jarde_reader::error::Error>>(
    child_definition: &PhysicalDefinitionId,
    root_name: &[u8],
    child: &ClassMemberFacts,
    budget: &mut Budget,
    mut analyze: impl FnMut(
        &PhysicalMethodId,
        &mut Budget,
    ) -> std::result::Result<jarde_jvm::method_ir::MethodIrAnalysis, F>,
) -> std::result::Result<std::result::Result<MemberCaptureProof, String>, F> {
    let mut analyses = Vec::new();
    for method in &child.methods {
        budget.poll()?;
        if !method
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Code")
        {
            continue;
        }
        let id = PhysicalMethodId {
            owner: child_definition.clone(),
            name: method.name.raw().clone(),
            descriptor: method.descriptor.raw().clone(),
        };
        let analyzed = analyze(&id, budget)?;
        if analyzed.report().method != id
            || !matches!(
                analyzed.report().execution,
                ExecutionReport::Complete { .. }
            )
        {
            return Ok(Err("member method SSA analysis did not complete".to_owned()));
        }
        analyses.push((id, analyzed));
    }
    let irs: Vec<_> = analyses
        .iter()
        .map(|(id, analyzed)| (id.clone(), analyzed.ir()))
        .collect();
    Ok(prove_family_capture(root_name, child, &irs, budget)?)
}

/// Prove a body-permitting instance capture from its ordered physical method demand.
/// Only the owned certificate leaves this scope; the analysis buffers end before fold consumers.
pub fn prove_instance_capture<F: From<jarde_reader::error::Error>>(
    child_definition: &PhysicalDefinitionId,
    root_name: &[u8],
    child: &ClassMemberFacts,
    budget: &mut Budget,
    mut analyze: impl FnMut(
        &PhysicalMethodId,
        &mut Budget,
    ) -> std::result::Result<jarde_jvm::method_ir::MethodIrAnalysis, F>,
) -> std::result::Result<std::result::Result<MemberCaptureProof, String>, F> {
    let mut analyses = Vec::new();
    for method in &child.methods {
        budget.poll()?;
        if !method
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Code")
        {
            continue;
        }
        let id = PhysicalMethodId {
            owner: child_definition.clone(),
            name: method.name.raw().clone(),
            descriptor: method.descriptor.raw().clone(),
        };
        let analyzed = analyze(&id, budget)?;
        if analyzed.report().method != id
            || !matches!(
                analyzed.report().execution,
                ExecutionReport::Complete { .. }
            )
        {
            return Ok(Err(
                "instance fold child method analysis did not complete".to_owned()
            ));
        }
        analyses.push((id, analyzed));
    }
    let irs: Vec<_> = analyses
        .iter()
        .map(|(id, analyzed)| (id.clone(), analyzed.ir()))
        .collect();
    match prove_family_instance_capture(root_name, child, &irs, budget)? {
        Ok(proof) => Ok(Ok(proof)),
        Err(reason) => Ok(Err(format!("instance capture refused: {reason}"))),
    }
}

/// Close the selected anonymous class's complete capture demand before choosing its source kind.
/// A non-complete analysis refuses this shape. The caller adapter remains responsible for
/// original execution/diagnostics; a source-shape None does not encode that failure channel.
pub fn prove_anonymous_capture<F: From<jarde_reader::error::Error>>(
    child_definition: &PhysicalDefinitionId,
    outer_name: &[u8],
    child: &ClassMemberFacts,
    receiver: CaptureReadReceiver,
    budget: &mut Budget,
    mut analyze: impl FnMut(
        &PhysicalMethodId,
        &mut Budget,
    ) -> std::result::Result<jarde_jvm::method_ir::MethodIrAnalysis, F>,
) -> std::result::Result<Option<MemberCaptureProof>, F> {
    if child.stopped_at.is_some()
        || child.fields.len() as u64 != child.field_count
        || child.methods.len() as u64 != child.method_count
        || child.fields.len() != 1
        || child.methods.iter().any(|method| {
            !method
                .attributes
                .iter()
                .any(|attribute| attribute.name.raw().0 == b"Code")
        })
    {
        return Ok(None);
    }
    let mut analyses = Vec::with_capacity(child.methods.len());
    for method in &child.methods {
        budget.poll()?;
        let id = PhysicalMethodId {
            owner: child_definition.clone(),
            name: method.name.raw().clone(),
            descriptor: method.descriptor.raw().clone(),
        };
        let analyzed = analyze(&id, budget)?;
        if analyzed.report().method != id
            || !matches!(
                analyzed.report().execution,
                ExecutionReport::Complete { .. }
            )
        {
            return Ok(None);
        }
        analyses.push((id, analyzed));
    }
    let irs: Vec<_> = analyses
        .iter()
        .map(|(id, analyzed)| (id.clone(), analyzed.ir()))
        .collect();
    let result = if child.fields[0].descriptor.raw().0 == b"D" {
        prove_anonymous_double_capture(child, &irs, budget)?
    } else if child.fields[0].name.raw().0.starts_with(b"val$") {
        prove_anonymous_val_capture(child, &irs, receiver, budget)?
    } else {
        prove_family_capture(outer_name, child, &irs, budget)?
    };
    match result {
        Ok(proof) => Ok(Some(proof)),
        Err(_) => Ok(None),
    }
}
