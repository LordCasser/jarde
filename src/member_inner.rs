//! Physical target proof for a Java 8 non-static member constructor.
//!
//! This module knows only the selected target class bytes. The facade selects those bytes from the
//! request's environment; a method recovery consumes the resulting narrow fact, never a class name
//! inferred from `$` or a constructor's first parameter alone.

use crate::class_source::ClassSourceAssemblyContext;
use crate::class_source::{
    MemberCallProof, MemberCaptureProof, MemberCaptureRead, OuterSuperBridgeProof,
    OuterSuperCallProof,
};
use jarde_jvm::method_ir::{Definition, MethodIr, Slot, SsaTable, ValueId};
use jarde_reader::budget::Budget;
use jarde_reader::budget::CountedBudgetDimension;
use jarde_reader::classfile::{
    ClassMemberFacts, CpEntryFacts, CpEntryKind, DescriptorKind, MethodCodeFacts, attribute_facts,
    class_constant_pool, cp_class_name, cp_entry, descriptor_facts, method_code_facts,
};
use jarde_reader::model::{PhysicalDefinitionId, PhysicalMethodId};

/// Consume a successful `new@1` verdict only for the exact constructor in the proved family.
/// The rule has already closed the SSA instance, checked qualifier copies, sole consumer,
/// argument dependency/effect order and exception coverage. These physical anchors ensure that
/// its decision is about the selected child and can later be mapped without changing this body.
pub(crate) fn prove_family_call_site(
    caller: &PhysicalMethodId,
    ir: &MethodIr,
    record: &jarde_java::init::NewRecord,
    constructor: &PhysicalMethodId,
    child_name: &[u8],
) -> std::result::Result<MemberCallProof, String> {
    let refuse = |reason: &str| Err(reason.to_owned());
    if !record.presented {
        return Err(record.refusal.as_ref().map_or_else(
            || "new@1 refused the member allocation without a reason".to_owned(),
            |refusal| format!("{}: {}", refusal.code, refusal.message),
        ));
    }
    let (Some(code), Some(dup), Some(call_bci)) = (ir.code(), record.dup, record.constructor)
    else {
        return refuse("new@1 did not publish complete construction anchors");
    };
    let pool = ir.constant_pool();
    let Some(head_index) = code
        .instructions
        .iter()
        .position(|instruction| instruction.bci == record.head)
    else {
        return refuse("new@1 allocation BCI is absent from the physical method");
    };
    let Some([head, copy, qualifier, qualifier_copy, check, pop]) =
        code.instructions.get(head_index..head_index + 6)
    else {
        return refuse("member allocation lacks the contiguous checked qualifier");
    };
    if head.opcode != 0xbb
        || copy.opcode != 0x59
        || copy.bci != dup
        || qualifier_copy.opcode != 0x59
        || pop.opcode != 0x57
        || !cp_class_name(pool, head.constant_pool_index.unwrap_or(0))
            .is_ok_and(|name| name.0 == child_name)
    {
        return refuse("new@1 anchors do not identify the exact child allocation");
    }
    let Some(call) = code
        .instructions
        .iter()
        .find(|instruction| instruction.bci == call_bci)
    else {
        return refuse("new@1 constructor BCI is absent from the physical method");
    };
    if call.opcode != 0xb7
        || !matches!(cp_entry(pool, call.constant_pool_index.unwrap_or(0)).ok().map(|entry| &entry.kind),
            Some(CpEntryKind::MethodRef { owner, name, descriptor, .. })
                if owner.0 == child_name
                    && name.0 == constructor.name.0 && descriptor.0 == constructor.descriptor.0)
    {
        return refuse("member call does not name the selected physical constructor");
    }
    if check.opcode != 0xb8
        || !matches!(cp_entry(pool, check.constant_pool_index.unwrap_or(0)).ok().map(|entry| &entry.kind),
        Some(CpEntryKind::MethodRef { owner, name, descriptor, .. })
            if owner.0 == b"java/util/Objects" && name.0 == b"requireNonNull"
                && descriptor.0 == b"(Ljava/lang/Object;)Ljava/lang/Object;")
    {
        return refuse("member call has no exact early requireNonNull check");
    }
    let Some((&first, ordinary)) = record.arguments.split_first() else {
        return refuse("new@1 did not identify the physical outer argument");
    };
    if first != qualifier_copy.bci
        || ordinary
            .iter()
            .any(|bci| *bci <= pop.bci || *bci >= call_bci)
    {
        return refuse("member call argument anchors do not follow the null check");
    }
    Ok(MemberCallProof {
        caller: caller.clone(),
        allocation_bci: record.head,
        copy_bci: dup,
        qualifier_bci: qualifier.bci,
        null_check_bci: check.bci,
        null_pop_bci: pop.bci,
        constructor_bci: call_bci,
        constructor: constructor.clone(),
        ordinary_argument_bcis: ordinary.to_vec(),
    })
}

/// This proof is deliberately narrower than the existing public `prove_target`: it uses the
/// already selected member relation and accepts a package-private physical constructor.
pub(crate) fn prove_family_capture(
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
            budget,
        )? {
            Ok(method_reads) => reads.extend(method_reads),
            Err(reason) => return Ok(Err(reason)),
        }
    }
    let Some(field_name) = std::str::from_utf8(&field.name.raw().0).ok() else {
        return refuse("capture field name is not UTF-8");
    };
    Ok(Ok(MemberCaptureProof {
        field_index: *field_index as u64,
        field_name: field_name.to_owned(),
        constructor: constructor_id.clone(),
        write_bci: 2,
        reads,
    }))
}

#[allow(clippy::too_many_arguments)]
fn scan_capture_method_uses(
    method: &PhysicalMethodId,
    ir: &MethodIr,
    code: &MethodCodeFacts,
    ssa: &SsaTable,
    constructor: &PhysicalMethodId,
    owner: &[u8],
    name: &[u8],
    descriptor: &[u8],
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
            if !capture_write_is_unique_site(method, instruction.bci, constructor) {
                return refuse("capture field has an extra write");
            }
            continue;
        }
        let Some(access) = ssa_instruction(ssa, instruction.bci) else {
            return refuse("capture read has no SSA instruction");
        };
        if access.reads().len() != 1 || !value_from_this_load(ssa, access.reads()[0].1) {
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

fn capture_write_is_unique_site(
    method: &PhysicalMethodId,
    bci: u32,
    constructor: &PhysicalMethodId,
) -> bool {
    method == constructor && bci == 2
}

fn capture_constructor_shape(code: &MethodCodeFacts) -> bool {
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

fn ssa_instruction(ssa: &SsaTable, bci: u32) -> Option<&jarde_jvm::method_ir::SsaInstruction> {
    ssa.blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| instruction.bci() == bci)
}

fn value_from_entry_load(ssa: &SsaTable, value: ValueId, slot: Slot, load_bci: u32) -> bool {
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

fn value_from_this_load(ssa: &SsaTable, value: ValueId) -> bool {
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return false;
    };
    value_from_entry_load(ssa, value, Slot::Local(0), *bci)
        && ssa_instruction(ssa, *bci).is_some_and(|load| load.opcode() == 0x2a)
}

/// Prove a single, exact physical bridge. Requiring a straight-line body also rules out extra
/// effects, hidden exits and a handler whose coverage a source-level call could change.
pub(crate) fn prove_outer_super_bridge(
    outer: &ClassMemberFacts,
    superclass: &ClassMemberFacts,
    superclass_definition: &PhysicalDefinitionId,
    bridge: &PhysicalMethodId,
    ir: &MethodIr,
    budget: &mut Budget,
) -> Result<std::result::Result<OuterSuperBridgeProof, String>> {
    let Some(code) = ir.code() else {
        return Ok(Err("bridge bytecode is unavailable".to_owned()));
    };
    prove_outer_super_bridge_body(
        outer,
        superclass,
        superclass_definition,
        bridge,
        ir,
        code,
        budget,
    )
}

fn prove_outer_super_bridge_body(
    outer: &ClassMemberFacts,
    superclass: &ClassMemberFacts,
    superclass_definition: &PhysicalDefinitionId,
    bridge: &PhysicalMethodId,
    ir: &MethodIr,
    code: &MethodCodeFacts,
    budget: &mut Budget,
) -> Result<std::result::Result<OuterSuperBridgeProof, String>> {
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    budget.poll()?;
    if outer.stopped_at.is_some()
        || outer.methods.len() as u64 != outer.method_count
        || bridge.name.0 == b"<init>"
        || bridge.name.0 == b"<clinit>"
    {
        return refuse("Outer method table or bridge identity is incomplete");
    }
    let definitions: Vec<_> = outer
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0 == bridge.name.0 && method.descriptor.raw().0 == bridge.descriptor.0
        })
        .collect();
    let [method] = definitions.as_slice() else {
        return refuse("bridge has no unique physical definition in selected Outer");
    };
    if !ir.declaration().is_some_and(|declaration| {
        declaration.identity() == bridge && declaration.class_name().0 == outer.this_class.raw().0
    }) || method.access_flags & (0x1000 | 0x0008) != 0x1008
        || method.access_flags & (0x0400 | 0x0100) != 0
    {
        return refuse("bridge is not the selected Outer synthetic static method");
    }
    let Some(super_name) = outer.super_class.as_ref() else {
        return refuse("Outer has no direct superclass");
    };
    if superclass.stopped_at.is_some()
        || superclass.methods.len() as u64 != superclass.method_count
        || superclass.this_class.raw().0 != super_name.raw().0
        || superclass.access_flags & 0x0200 != 0
    {
        return refuse("selected direct superclass definition is incomplete or mismatched");
    }
    let descriptor = &bridge.descriptor.0;
    let Ok(signature) = descriptor_facts(descriptor, DescriptorKind::Method) else {
        return refuse("bridge descriptor is invalid");
    };
    let Some((receiver, ordinary)) = signature.parameters().split_first() else {
        return refuse("bridge has no Outer receiver parameter");
    };
    if receiver.bytes(descriptor)
        != Some(
            [b"L".as_slice(), &outer.this_class.raw().0, b";"]
                .concat()
                .as_slice(),
        )
    {
        return refuse("bridge first parameter is not the selected Outer");
    }
    let Some(ssa) = ir.ssa() else {
        return refuse("bridge SSA is unavailable");
    };
    if code.stopped_at.is_some()
        || !code.exception_handlers.is_empty()
        || code.instructions.len() != ordinary.len() + 3
    {
        return refuse("bridge has an extra instruction, exit or exception handler");
    }
    let mut slot = 0u16;
    let mut load_values = Vec::with_capacity(ordinary.len() + 1);
    for (index, component) in signature.parameters().iter().enumerate() {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let instruction = &code.instructions[index];
        let Some(load) = ssa_instruction(ssa, instruction.bci) else {
            return refuse("bridge parameter load has no SSA instruction");
        };
        let Some(type_bytes) = component.bytes(descriptor) else {
            return refuse("bridge parameter descriptor span is invalid");
        };
        if !bridge_load_opcode(instruction.opcode, type_bytes)
            || load.reads().len() != 1
            || load.writes().len() != 1
            || load.reads()[0].0 != Slot::Local(slot)
            || !value_from_entry_load(ssa, load.writes()[0].1, Slot::Local(slot), instruction.bci)
        {
            return refuse("bridge does not load its receiver and parameters in descriptor order");
        }
        load_values.push(load.writes()[0].1);
        slot = slot.saturating_add(component.slots());
    }
    let invoke = &code.instructions[ordinary.len() + 1];
    let returns = &code.instructions[ordinary.len() + 2];
    if invoke.opcode != 0xb7
        || !bridge_return_opcode(
            returns.opcode,
            signature.result().and_then(|part| part.bytes(descriptor)),
        )
    {
        return refuse("bridge has no single matching invokespecial and return");
    }
    let Some(call) = ssa_instruction(ssa, invoke.bci) else {
        return refuse("bridge invokespecial has no SSA instruction");
    };
    if call.reads().len() != load_values.len()
        || call
            .reads()
            .iter()
            .map(|(_, value)| *value)
            .ne(load_values.iter().rev().copied())
    {
        return refuse("bridge invokespecial reorders or replaces parameters");
    }
    let Some(exit) = ssa_instruction(ssa, returns.bci) else {
        return refuse("bridge return has no SSA instruction");
    };
    if signature.result().is_some() {
        if call.writes().len() != 1
            || exit.reads().len() != 1
            || call.writes()[0].1 != exit.reads()[0].1
        {
            return refuse("bridge return is not the invokespecial result");
        }
    } else if !call.writes().is_empty() || !exit.reads().is_empty() {
        return refuse("void bridge has an unexpected return value");
    }
    let Some(index) = invoke.constant_pool_index else {
        return refuse("bridge invokespecial has no MethodRef");
    };
    let Ok(entry) = cp_entry(ir.constant_pool(), index) else {
        return refuse("bridge invokespecial MethodRef is unreadable");
    };
    let CpEntryKind::MethodRef {
        owner,
        name,
        descriptor: target_descriptor,
        ..
    } = &entry.kind
    else {
        return refuse("bridge invokespecial does not name a class method");
    };
    let mut expected_target = vec![b'('];
    for parameter in ordinary {
        let Some(bytes) = parameter.bytes(descriptor) else {
            return refuse("bridge target parameter descriptor span is invalid");
        };
        expected_target.extend_from_slice(bytes);
    }
    expected_target.push(b')');
    expected_target.extend_from_slice(
        signature
            .result()
            .and_then(|part| part.bytes(descriptor))
            .unwrap_or(b"V"),
    );
    if owner.0 != super_name.raw().0
        || name.0 == b"<init>"
        || name.0 == b"<clinit>"
        || target_descriptor.0 != expected_target
    {
        return refuse("bridge target is not the direct superclass method with ordered parameters");
    }
    let targets: Vec<_> = superclass
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0 == name.0 && method.descriptor.raw().0 == target_descriptor.0
        })
        .collect();
    let [target] = targets.as_slice() else {
        return refuse("direct superclass has no unique exact bridge target definition");
    };
    if target.access_flags & (0x0002 | 0x0008 | 0x0400) != 0 {
        return refuse("direct superclass target is private, static or abstract");
    }
    let outer_package = outer
        .this_class
        .raw()
        .0
        .rsplitn(2, |byte| *byte == b'/')
        .nth(1);
    let super_package = superclass
        .this_class
        .raw()
        .0
        .rsplitn(2, |byte| *byte == b'/')
        .nth(1);
    if target.access_flags & (0x0001 | 0x0004) == 0 && outer_package != super_package {
        return refuse("direct superclass target is not accessible from Outer source");
    }
    Ok(Ok(OuterSuperBridgeProof {
        bridge: bridge.clone(),
        outer_name: outer.this_class.raw().clone(),
        invoke_bci: invoke.bci,
        target_method: PhysicalMethodId {
            owner: superclass_definition.clone(),
            name: name.clone(),
            descriptor: target_descriptor.clone(),
        },
        target_owner: owner.clone(),
        target_name: name.clone(),
        target_descriptor: target_descriptor.clone(),
    }))
}

fn bridge_load_opcode(opcode: u8, ty: &[u8]) -> bool {
    let base = match ty.first().copied() {
        Some(b'J') => 0x16,
        Some(b'F') => 0x17,
        Some(b'D') => 0x18,
        Some(b'L' | b'[') => 0x19,
        Some(b'B' | b'C' | b'I' | b'S' | b'Z') => 0x15,
        _ => return false,
    };
    let implicit = match base {
        0x15 => 0x1a,
        0x16 => 0x1e,
        0x17 => 0x22,
        0x18 => 0x26,
        _ => 0x2a,
    };
    opcode == base || (implicit..=implicit + 3).contains(&opcode)
}

fn bridge_return_opcode(opcode: u8, ty: Option<&[u8]>) -> bool {
    match ty.and_then(|ty| ty.first().copied()) {
        None => opcode == 0xb1,
        Some(b'J') => opcode == 0xad,
        Some(b'F') => opcode == 0xae,
        Some(b'D') => opcode == 0xaf,
        Some(b'L' | b'[') => opcode == 0xb0,
        Some(b'B' | b'C' | b'I' | b'S' | b'Z') => opcode == 0xac,
        _ => false,
    }
}

/// Bind one `invokestatic` operand to the *value* read from the certified capture field.
/// SSA operands of an invocation are recorded in pop order, so the receiver is last.
pub(crate) fn prove_outer_super_call(
    caller: &PhysicalMethodId,
    ir: &MethodIr,
    capture: &MemberCaptureProof,
    bridge: &OuterSuperBridgeProof,
    call_bci: u32,
    budget: &mut Budget,
) -> Result<std::result::Result<OuterSuperCallProof, String>> {
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    budget.poll()?;
    if !ir
        .declaration()
        .is_some_and(|declaration| declaration.identity() == caller)
        || caller.owner != capture.constructor.owner
    {
        return refuse("caller is not a method of the proved member definition");
    }
    let (Some(code), Some(ssa)) = (ir.code(), ir.ssa()) else {
        return refuse("caller code or SSA is unavailable");
    };
    if code.stopped_at.is_some() {
        return refuse("caller bytecode is incomplete");
    }
    let Some(instruction) = code
        .instructions
        .iter()
        .find(|instruction| instruction.bci == call_bci)
    else {
        return refuse("bridge call BCI is absent from the physical caller");
    };
    if instruction.opcode != 0xb8
        || !matches!(instruction.constant_pool_index.and_then(|index| cp_entry(ir.constant_pool(), index).ok()).map(|entry| &entry.kind),
            Some(CpEntryKind::MethodRef { owner, name, descriptor, .. })
                if owner == &bridge.outer_name
                    && name == &bridge.bridge.name && descriptor == &bridge.bridge.descriptor)
    {
        return refuse("call does not name the exact static Outer bridge");
    }
    let Some(call) = ssa_instruction(ssa, call_bci) else {
        return refuse("bridge call has no SSA instruction");
    };
    let Ok(signature) = descriptor_facts(&bridge.bridge.descriptor.0, DescriptorKind::Method)
    else {
        return refuse("bridge descriptor cannot be parsed at call site");
    };
    if call.reads().len() != signature.parameters().len() {
        return refuse("bridge call SSA arity differs from its descriptor");
    }
    let Some(read_value) = call.reads().last().map(|(_, value)| *value) else {
        return refuse("bridge call has no receiver operand");
    };
    let Definition::Instruction {
        bci: capture_read_bci,
        ..
    } = ssa.value(read_value).def()
    else {
        return refuse("bridge receiver is not a direct capture SSA read");
    };
    if !capture.reads.iter().any(|read| {
        read.method == *caller
            && read.bci == *capture_read_bci
            && read.consumer_bcis.contains(&call_bci)
    }) {
        return refuse("bridge receiver comes from another Outer value, not the proved capture");
    }
    let Some(field_read) = ssa_instruction(ssa, *capture_read_bci) else {
        return refuse("capture read SSA instruction is missing");
    };
    if field_read.opcode() != 0xb4
        || field_read.writes().len() != 1
        || field_read.writes()[0].1 != read_value
        || *capture_read_bci >= call_bci
    {
        return refuse("bridge receiver is not the earlier captured Outer field value");
    }
    let mut argument_bcis = Vec::new();
    let mut argument_dependencies = std::collections::BTreeSet::new();
    let mut previous = *capture_read_bci;
    for (_, value) in call.reads().iter().rev().skip(1) {
        let Some(dependencies) =
            ordered_argument_dependencies(ssa, *value, *capture_read_bci, call_bci, budget)?
        else {
            return refuse("bridge argument has an unproved SSA dependency");
        };
        let Some((&first, &last)) = dependencies.first().zip(dependencies.last()) else {
            return refuse("bridge argument has no physical producer after capture");
        };
        if first <= previous
            || last >= call_bci
            || dependencies
                .iter()
                .any(|bci| !argument_dependencies.insert(*bci))
        {
            return refuse("bridge arguments have overlapping or reordered effects");
        }
        previous = last;
        argument_bcis.push(last);
    }
    let Some(read_index) = code
        .instructions
        .iter()
        .position(|instruction| instruction.bci == *capture_read_bci)
    else {
        return refuse("capture read is absent from caller bytecode");
    };
    let Some(call_index) = code
        .instructions
        .iter()
        .position(|instruction| instruction.bci == call_bci)
    else {
        return refuse("bridge call is absent from caller bytecode");
    };
    let Some(argument_instructions) = code.instructions.get(read_index + 1..call_index) else {
        return refuse("bridge argument instruction range is invalid");
    };
    if argument_instructions.len() != argument_dependencies.len()
        || argument_instructions.iter().any(|instruction| {
            !argument_dependencies.contains(&instruction.bci)
                || matches!(
                    instruction.opcode,
                    0x99..=0xa9 | 0xaa | 0xab | 0xc6 | 0xc7 | 0xc8 | 0xc9 | 0xbf
                )
        })
    {
        return refuse("bridge argument range contains an unrelated effect or control-flow exit");
    }
    // Moving the call into the source-level `Outer.super` expression must not move it across a
    // handler boundary. This also keeps any argument effect in the same exception region.
    for bci in std::iter::once(*capture_read_bci).chain(argument_dependencies.iter().copied()) {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if code.exception_handlers.iter().any(|handler| {
            (handler.start_bci <= bci && bci < handler.end_bci)
                != (handler.start_bci <= call_bci && call_bci < handler.end_bci)
        }) {
            return refuse("bridge expression crosses an exception handler boundary");
        }
    }
    Ok(Ok(OuterSuperCallProof {
        caller: caller.clone(),
        call_bci,
        capture_read_bci: *capture_read_bci,
        argument_bcis,
        bridge: bridge.clone(),
    }))
}

fn ordered_argument_dependencies(
    ssa: &SsaTable,
    value: ValueId,
    capture_bci: u32,
    call_bci: u32,
    budget: &mut Budget,
) -> Result<Option<std::collections::BTreeSet<u32>>> {
    let mut pending = vec![value];
    let mut seen = std::collections::HashSet::new();
    let mut dependencies = std::collections::BTreeSet::new();
    while let Some(value) = pending.pop() {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if !seen.insert(value) {
            continue;
        }
        match ssa.value(value).def() {
            Definition::Entry { .. } => {}
            Definition::Instruction { bci, .. } if *bci > capture_bci && *bci < call_bci => {
                let Some(instruction) = ssa_instruction(ssa, *bci) else {
                    return Ok(None);
                };
                dependencies.insert(*bci);
                pending.extend(instruction.reads().iter().map(|(_, input)| *input));
            }
            _ => return Ok(None),
        }
    }
    Ok(Some(dependencies))
}

/// A class's own typed row is only a candidate until the selected child's row agrees.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FamilyRootCandidate {
    pub(crate) child_name: Vec<u8>,
    pub(crate) simple_name: String,
    pub(crate) access_flags: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FamilyRootScan {
    Absent,
    Refused(String),
    Candidate(FamilyRootCandidate),
}

const FAMILY_FORBIDDEN_FLAGS: u16 = 0x0008 | 0x0200 | 0x2000 | 0x4000;
const FAMILY_VISIBILITY_FLAGS: u16 = 0x0001 | 0x0002 | 0x0004;

/// Discover at most one direct named non-static child from the root's typed InnerClasses rows.
/// No binary-name search is used: the class index in the row supplies the exact symbolic target.
pub(crate) fn scan_family_root(
    root: &[u8],
    nesting: &ClassSourceAssemblyContext,
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> Result<FamilyRootScan> {
    if nesting.enclosing_method.is_some() {
        return Ok(FamilyRootScan::Refused(
            "selected root has EnclosingMethod identity".to_owned(),
        ));
    }
    let mut candidate = None;
    let mut static_names = Vec::new();
    for row in &nesting.inner_classes {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        // This slice writes one top-level source unit. A typed self row with an outer owner
        // proves that the selected root itself belongs to a larger family.
        if row.outer_class_index != 0
            && cp_class_name(pool, row.class_index).is_ok_and(|name| name.0 == root)
        {
            return Ok(FamilyRootScan::Refused(
                "selected root has an InnerClasses self row naming an outer class".to_owned(),
            ));
        }
        if row.outer_class_index == 0 {
            continue;
        }
        let Ok(outer) = cp_class_name(pool, row.outer_class_index) else {
            return Ok(FamilyRootScan::Refused(
                "InnerClasses outer class index is invalid".to_owned(),
            ));
        };
        if outer.0 != root {
            continue;
        }
        // Static nested declarations do not join this captured-member subset. Retain their
        // identities so a contradictory static row for the selected child still rejects it.
        if row.access_flags & 0x0008 != 0 {
            let Ok(name) = cp_class_name(pool, row.class_index) else {
                return Ok(FamilyRootScan::Refused(
                    "static nested class index is invalid".to_owned(),
                ));
            };
            static_names.push(name.0);
            continue;
        }
        let Ok(child) = cp_class_name(pool, row.class_index) else {
            return Ok(FamilyRootScan::Refused(
                "direct member class index is invalid".to_owned(),
            ));
        };
        let Some(simple) = row
            .inner_name
            .as_ref()
            .and_then(|name| std::str::from_utf8(&name.0).ok())
        else {
            return Ok(FamilyRootScan::Refused(
                "direct member has no UTF-8 source name".to_owned(),
            ));
        };
        if !jarde_java::names::is_java_identifier(simple)
            || child.0 != [root, b"$", simple.as_bytes()].concat()
            || row.access_flags & FAMILY_FORBIDDEN_FLAGS != 0
            || (row.access_flags & FAMILY_VISIBILITY_FLAGS).count_ones() > 1
            || row.access_flags & (0x0010 | 0x0400) == (0x0010 | 0x0400)
        {
            return Ok(FamilyRootScan::Refused(
                "direct member is not a source-spellable named non-static class".to_owned(),
            ));
        }
        let next = FamilyRootCandidate {
            child_name: child.0,
            simple_name: simple.to_owned(),
            access_flags: row.access_flags,
        };
        if candidate.replace(next).is_some() {
            return Ok(FamilyRootScan::Refused(
                "multiple direct member rows are outside the one-child family subset".to_owned(),
            ));
        }
    }
    if candidate
        .as_ref()
        .is_some_and(|selected: &FamilyRootCandidate| static_names.contains(&selected.child_name))
    {
        return Ok(FamilyRootScan::Refused(
            "selected member also has a conflicting static InnerClasses row".to_owned(),
        ));
    }
    Ok(candidate.map_or(FamilyRootScan::Absent, FamilyRootScan::Candidate))
}

/// The selected child must independently state the identical unique self relation.
pub(crate) fn child_relation_agrees(
    root: &[u8],
    candidate: &FamilyRootCandidate,
    child: &ClassMemberFacts,
    nesting: &ClassSourceAssemblyContext,
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> Result<bool> {
    if child.stopped_at.is_some()
        || child.this_class.raw().0 != candidate.child_name
        || nesting.enclosing_method.is_some()
    {
        return Ok(false);
    }
    let mut self_row = None;
    for row in &nesting.inner_classes {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let Ok(name) = cp_class_name(pool, row.class_index) else {
            return Ok(false);
        };
        if name.0 == candidate.child_name && self_row.replace(row).is_some() {
            return Ok(false);
        }
    }
    let Some(row) = self_row else {
        return Ok(false);
    };
    Ok(row.outer_class_index != 0
        && cp_class_name(pool, row.outer_class_index).is_ok_and(|outer| outer.0 == root)
        && row
            .inner_name
            .as_ref()
            .is_some_and(|name| name.0 == candidate.simple_name.as_bytes())
        && row.access_flags == candidate.access_flags)
}
use jarde_reader::error::{Error, Result};
use jarde_reader::signature::{
    SignatureType, parse_class_signature, parse_method_signature, prove_class_signature_erasure,
    prove_method_signature_erasure_with_class_scope,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MemberInnerTarget {
    pub(crate) owner: String,
    pub(crate) outer: String,
    pub(crate) simple_name: String,
    pub(crate) constructor_descriptor: String,
    pub(crate) capture_field: String,
    pub(crate) generic_diamond: bool,
    pub(crate) source_type_parameter_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GenericOuterRelationProof {
    pub(crate) enclosing: String,
    pub(crate) simple_name: String,
    pub(crate) type_parameter_count: usize,
    /// The selected owner itself is the top-level, non-generic class that declares the target.
    /// This preserves the previously proved `Outer.Inner` source path without treating a `$` in a
    /// binary name as a nested source path.
    pub(crate) top_level_non_generic_owner: bool,
}

fn refusal_or_stop<T>(error: Error) -> Result<Option<T>> {
    match error {
        Error::BudgetExceeded { .. } | Error::Cancelled { .. } => Err(error),
        _ => Ok(None),
    }
}

/// Prove one exact constructor in the target definition selected by the facade. `None` means one
/// structural premise was absent; a request-ending budget/cancellation remains an error.
pub(crate) fn prove_target(
    bytes: &[u8],
    target: &ClassMemberFacts,
    owner: &str,
    descriptor: &str,
    budget: &mut Budget,
) -> Result<Option<MemberInnerTarget>> {
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_STATIC: u16 = 0x0008;
    const ACC_INTERFACE: u16 = 0x0200;
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_SYNTHETIC: u16 = 0x1000;
    const ACC_ANNOTATION: u16 = 0x2000;
    const ACC_ENUM: u16 = 0x4000;

    budget.poll()?;
    if target.stopped_at.is_some()
        || target.this_class.raw().0.as_slice() != owner.as_bytes()
        || target.access_flags
            & (ACC_PUBLIC | ACC_INTERFACE | ACC_ABSTRACT | ACC_ANNOTATION | ACC_ENUM)
            != ACC_PUBLIC
    {
        return Ok(None);
    }
    let pool = match class_constant_pool(bytes, budget) {
        Ok(pool) => pool,
        Err(error) => return refusal_or_stop(error),
    };
    let nesting_shells: Vec<_> = target
        .attributes
        .iter()
        .filter(|attribute| {
            matches!(
                attribute.name.raw().0.as_slice(),
                b"InnerClasses" | b"EnclosingMethod"
            )
        })
        .cloned()
        .collect();
    let nesting = match attribute_facts(bytes, &nesting_shells, &pool, budget) {
        Ok(nesting) => nesting,
        Err(error) => return refusal_or_stop(error),
    };
    if nesting.enclosing_method.is_some() {
        return Ok(None);
    }
    let self_entries: Vec<_> = nesting
        .inner_classes
        .iter()
        .filter(|entry| {
            cp_class_name(&pool, entry.class_index).is_ok_and(|name| name.0 == owner.as_bytes())
        })
        .collect();
    let [relation] = self_entries.as_slice() else {
        return Ok(None);
    };
    if relation.outer_class_index == 0
        || relation.access_flags & (ACC_PUBLIC | ACC_STATIC) != ACC_PUBLIC
    {
        return Ok(None);
    }
    let Ok(outer) = cp_class_name(&pool, relation.outer_class_index) else {
        return Ok(None);
    };
    let (Ok(outer), Some(simple_name)) = (
        std::str::from_utf8(&outer.0),
        relation
            .inner_name
            .as_ref()
            .and_then(|name| std::str::from_utf8(&name.0).ok()),
    ) else {
        return Ok(None);
    };
    if !jarde_java::names::is_java_identifier(simple_name)
        || owner != format!("{outer}${simple_name}")
        || descriptor == "()V"
        || !descriptor.starts_with(&format!("(L{outer};"))
        || !descriptor.ends_with(")V")
        || descriptor_facts(descriptor.as_bytes(), DescriptorKind::Method).is_err()
    {
        return Ok(None);
    }

    let class_signature_shells: Vec<_> = target
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"Signature")
        .cloned()
        .collect();
    let class_type_annotations_present = target.attributes.iter().any(|attribute| {
        matches!(
            attribute.name.raw().0.as_slice(),
            b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
        )
    });
    let class_attributes = match attribute_facts(bytes, &class_signature_shells, &pool, budget) {
        Ok(attributes) => attributes,
        Err(error) => return refusal_or_stop(error),
    };
    let class_scope = match class_attributes.signature {
        None => None,
        Some(raw_signature) => {
            if class_type_annotations_present {
                return Ok(None);
            }
            let parsed = match parse_class_signature(&raw_signature.0, budget) {
                Ok(signature) => signature,
                Err(error) => return refusal_or_stop(error),
            };
            if parsed.type_parameters.len() != 1
                || parsed.type_parameters[0].name != b"V"
                || parsed.type_parameters[0]
                    .class_bound
                    .as_ref()
                    .is_none_or(|bound| {
                        !matches!(
                            bound,
                            SignatureType::Class(class)
                                if class.segments.len() == 1
                                    && class.segments[0].binary_name == b"java/lang/Object"
                                    && class.segments[0].arguments.is_empty()
                        )
                    })
                || !parsed.type_parameters[0].interface_bounds.is_empty()
                || !parsed.interfaces.is_empty()
                || !target.interfaces.is_empty()
                || parsed.superclass.segments.len() != 1
                || parsed.superclass.segments[0].binary_name != b"java/lang/Object"
                || !parsed.superclass.segments[0].arguments.is_empty()
            {
                return Ok(None);
            }
            let Some(superclass) = target.super_class.as_ref() else {
                return Ok(None);
            };
            let physical_interfaces: Vec<Vec<u8>> = target
                .interfaces
                .iter()
                .map(|interface| interface.raw().0.clone())
                .collect();
            let proof = match prove_class_signature_erasure(
                &parsed,
                &superclass.raw().0,
                &physical_interfaces,
                budget,
            ) {
                Ok(proof) => proof,
                Err(error) => return refusal_or_stop(error),
            };
            if proof.type_parameters.len() != 1
                || proof.type_parameters[0].name != b"V"
                || proof.type_parameters[0].descriptor != b"Ljava/lang/Object;"
            {
                return Ok(None);
            }
            Some(proof.type_parameters)
        }
    };

    let constructors: Vec<_> = target
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0 == b"<init>" && method.descriptor.raw().0 == descriptor.as_bytes()
        })
        .collect();
    let [constructor] = constructors.as_slice() else {
        return Ok(None);
    };
    if constructor.access_flags & ACC_PUBLIC == 0
        || constructor.access_flags & ACC_STATIC != 0
        || constructor
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"Code")
            .count()
            != 1
    {
        return Ok(None);
    }
    let constructor_signatures: Vec<_> = constructor
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"Signature")
        .cloned()
        .collect();
    let constructor_type_annotations_present = constructor.attributes.iter().any(|attribute| {
        matches!(
            attribute.name.raw().0.as_slice(),
            b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
        )
    });
    let constructor_attributes =
        match attribute_facts(bytes, &constructor_signatures, &pool, budget) {
            Ok(attributes) => attributes,
            Err(error) => return refusal_or_stop(error),
        };
    let outer_descriptor = format!("L{outer};");
    let generic_diamond = match (class_scope.as_deref(), constructor_attributes.signature) {
        (None, None) => false,
        (Some(class_scope), Some(raw_signature)) => {
            let all_constructors: Vec<_> = target
                .methods
                .iter()
                .filter(|method| method.name.raw().0 == b"<init>")
                .collect();
            if all_constructors.len() != 1 || constructor_type_annotations_present {
                return Ok(None);
            }
            let parsed = match parse_method_signature(&raw_signature.0, budget) {
                Ok(signature) => signature,
                Err(error) => return refusal_or_stop(error),
            };
            if !parsed.type_parameters.is_empty()
                || parsed.parameters.as_slice() != [SignatureType::TypeVariable(b"V".to_vec())]
                || parsed.result.is_some()
                || !parsed.throws.is_empty()
            {
                return Ok(None);
            }
            let physical = match descriptor_facts(descriptor.as_bytes(), DescriptorKind::Method) {
                Ok(physical) => physical,
                Err(_) => return Ok(None),
            };
            let Some(first) = physical.parameters().first() else {
                return Ok(None);
            };
            if first.bytes(descriptor.as_bytes()) != Some(outer_descriptor.as_bytes()) {
                return Ok(None);
            }
            let mut source_descriptor = b"(".to_vec();
            for component in &physical.parameters()[1..] {
                let Some(bytes) = component.bytes(descriptor.as_bytes()) else {
                    return Ok(None);
                };
                source_descriptor.extend_from_slice(bytes);
            }
            source_descriptor.extend_from_slice(b")V");
            let erasure = match prove_method_signature_erasure_with_class_scope(
                &parsed,
                &source_descriptor,
                &[],
                class_scope,
                budget,
            ) {
                Ok(erasure) => erasure,
                Err(error) => return refusal_or_stop(error),
            };
            if erasure.parameters.as_slice() != [b"Ljava/lang/Object;".to_vec()]
                || erasure.result.is_some()
            {
                return Ok(None);
            }
            true
        }
        _ => return Ok(None),
    };
    let source_type_parameter_count = class_scope.as_ref().map_or(0, Vec::len);
    let captures: Vec<_> = target
        .fields
        .iter()
        .filter(|field| {
            field.access_flags & ACC_SYNTHETIC != 0 && field.access_flags & ACC_STATIC == 0
        })
        .collect();
    let [capture] = captures.as_slice() else {
        return Ok(None);
    };
    if capture.descriptor.raw().0 != outer_descriptor.as_bytes() {
        return Ok(None);
    }
    let code = match method_code_facts(bytes, constructor, budget) {
        Ok(code) => code,
        Err(error) => return refusal_or_stop(error),
    };
    if code.stopped_at.is_some() || code.instructions.len() < 3 {
        return Ok(None);
    }
    let prologue = &code.instructions[..3];
    if prologue[0].bci != 0
        || prologue[0].opcode != 0x2a // aload_0: uninitialized this
        || prologue[1].bci != 1
        || prologue[1].opcode != 0x2b // aload_1: physical first parameter
        || prologue[2].bci != 2
        || prologue[2].opcode != 0xb5 // putfield: capture on this
        || prologue[2].constant_pool_index.is_none()
    {
        return Ok(None);
    }
    let field_index = prologue[2].constant_pool_index.expect("checked above");
    let Ok(entry) = cp_entry(&pool, field_index) else {
        return Ok(None);
    };
    if !matches!(
        &entry.kind,
        CpEntryKind::FieldRef { owner: field_owner, name, descriptor: field_descriptor, .. }
            if field_owner.0 == owner.as_bytes()
                && name.0 == capture.name.raw().0
                && field_descriptor.0 == outer_descriptor.as_bytes()
    ) {
        return Ok(None);
    }
    let Ok(capture_field) = std::str::from_utf8(&capture.name.raw().0) else {
        return Ok(None);
    };
    Ok(Some(MemberInnerTarget {
        owner: owner.to_owned(),
        outer: outer.to_owned(),
        simple_name: simple_name.to_owned(),
        constructor_descriptor: descriptor.to_owned(),
        capture_field: capture_field.to_owned(),
        generic_diamond,
        source_type_parameter_count,
    }))
}

/// A qualified Java creation must also resolve `Inner` through the selected outer class. The
/// target's self row alone does not make that member visible to `javac`: the outer's matching
/// InnerClasses row is a separate physical premise.
pub(crate) fn outer_relation_agrees(
    bytes: &[u8],
    outer_class: &ClassMemberFacts,
    target: &MemberInnerTarget,
    budget: &mut Budget,
) -> Result<Option<GenericOuterRelationProof>> {
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_PRIVATE: u16 = 0x0002;
    const ACC_PROTECTED: u16 = 0x0004;
    const ACC_STATIC: u16 = 0x0008;
    const ACC_INTERFACE: u16 = 0x0200;
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_ANNOTATION: u16 = 0x2000;
    const ACC_ENUM: u16 = 0x4000;

    budget.poll()?;
    if outer_class.stopped_at.is_some()
        || outer_class.this_class.raw().0 != target.outer.as_bytes()
        || outer_class.access_flags
            & (ACC_PUBLIC | ACC_INTERFACE | ACC_ABSTRACT | ACC_ANNOTATION | ACC_ENUM)
            != ACC_PUBLIC
    {
        return Ok(None);
    }
    let pool = match class_constant_pool(bytes, budget) {
        Ok(pool) => pool,
        Err(error) => return refusal_or_stop(error),
    };
    let shells: Vec<_> = outer_class
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"InnerClasses")
        .cloned()
        .collect();
    let mut nesting_shells: Vec<_> = shells;
    nesting_shells.extend(
        outer_class
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"EnclosingMethod")
            .cloned(),
    );
    let nesting = match attribute_facts(bytes, &nesting_shells, &pool, budget) {
        Ok(nesting) => nesting,
        Err(error) => return refusal_or_stop(error),
    };
    if nesting.enclosing_method.is_some() {
        return Ok(None);
    }

    let target_rows: Vec<_> = nesting
        .inner_classes
        .iter()
        .filter(|entry| {
            cp_class_name(&pool, entry.class_index)
                .is_ok_and(|name| name.0 == target.owner.as_bytes())
        })
        .collect();
    let [target_row] = target_rows.as_slice() else {
        return Ok(None);
    };
    if target_row.outer_class_index == 0
        || target_row.access_flags & (ACC_PUBLIC | ACC_PRIVATE | ACC_PROTECTED | ACC_STATIC)
            != ACC_PUBLIC
        || !cp_class_name(&pool, target_row.outer_class_index)
            .is_ok_and(|name| name.0 == target.outer.as_bytes())
        || !target_row
            .inner_name
            .as_ref()
            .is_some_and(|name| name.0 == target.simple_name.as_bytes())
    {
        return Ok(None);
    }

    // `A<T>` is a public static member of its selected `Outer`. Its own row supplies the candidate
    // name; the reciprocal row on the selected `Outer` is checked by the facade below.
    let self_rows: Vec<_> = nesting
        .inner_classes
        .iter()
        .filter(|entry| {
            cp_class_name(&pool, entry.class_index)
                .is_ok_and(|name| name.0 == target.outer.as_bytes())
        })
        .collect();
    if self_rows.is_empty() {
        // Keep the accepted non-generic `Outer.Inner` slice. A binary `$` by itself cannot prove
        // that the selected owner is a top-level source declaration, so this branch requires a
        // simple top-level binary name, no class Signature, and the exact reciprocal member row
        // already checked above.
        let owner_simple_name = target.outer.rsplit('/').next().unwrap_or_default();
        if owner_simple_name.is_empty()
            || owner_simple_name.contains('$')
            || !jarde_java::names::is_java_identifier(owner_simple_name)
            || outer_class
                .attributes
                .iter()
                .any(|attribute| attribute.name.raw().0 == b"Signature")
        {
            return Ok(None);
        }
        return Ok(Some(GenericOuterRelationProof {
            enclosing: target.outer.clone(),
            simple_name: String::new(),
            type_parameter_count: 0,
            top_level_non_generic_owner: true,
        }));
    }
    let [self_row] = self_rows.as_slice() else {
        return Ok(None);
    };
    if self_row.outer_class_index == 0
        || self_row.access_flags & (ACC_PUBLIC | ACC_PRIVATE | ACC_PROTECTED | ACC_STATIC)
            != (ACC_PUBLIC | ACC_STATIC)
    {
        return Ok(None);
    }
    let Ok(enclosing_class) = cp_class_name(&pool, self_row.outer_class_index) else {
        return Ok(None);
    };
    let (Ok(enclosing), Some(simple_name)) = (
        String::from_utf8(enclosing_class.0),
        self_row
            .inner_name
            .as_ref()
            .and_then(|name| std::str::from_utf8(&name.0).ok()),
    ) else {
        return Ok(None);
    };
    if !jarde_java::names::is_java_identifier(simple_name)
        || target.outer != format!("{enclosing}${simple_name}")
    {
        return Ok(None);
    }

    let signatures: Vec<_> = outer_class
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"Signature")
        .cloned()
        .collect();
    if signatures.len() != 1
        || outer_class.attributes.iter().any(|attribute| {
            matches!(
                attribute.name.raw().0.as_slice(),
                b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
            )
        })
    {
        return Ok(None);
    }
    let signature = match attribute_facts(bytes, &signatures, &pool, budget) {
        Ok(attributes) => attributes.signature,
        Err(error) => return refusal_or_stop(error),
    };
    let Some(signature) = signature else {
        return Ok(None);
    };
    let parsed = match parse_class_signature(&signature.0, budget) {
        Ok(signature) => signature,
        Err(error) => return refusal_or_stop(error),
    };
    if parsed.type_parameters.len() != 1
        || parsed.type_parameters[0].name != b"T"
        || !matches!(
            parsed.type_parameters[0].class_bound.as_ref(),
            Some(SignatureType::Class(class))
                if class.segments.len() == 1
                    && class.segments[0].binary_name == b"java/lang/Object"
                    && class.segments[0].arguments.is_empty()
        )
        || !parsed.type_parameters[0].interface_bounds.is_empty()
        || !parsed.interfaces.is_empty()
        || !outer_class.interfaces.is_empty()
        || parsed.superclass.segments.len() != 1
        || parsed.superclass.segments[0].binary_name != b"java/lang/Object"
        || !parsed.superclass.segments[0].arguments.is_empty()
    {
        return Ok(None);
    }
    let Some(superclass) = outer_class.super_class.as_ref() else {
        return Ok(None);
    };
    let physical_interfaces: Vec<Vec<u8>> = outer_class
        .interfaces
        .iter()
        .map(|interface| interface.raw().0.clone())
        .collect();
    let proof = match prove_class_signature_erasure(
        &parsed,
        &superclass.raw().0,
        &physical_interfaces,
        budget,
    ) {
        Ok(proof) => proof,
        Err(error) => return refusal_or_stop(error),
    };
    if proof.type_parameters.len() != 1
        || proof.type_parameters[0].name != b"T"
        || proof.type_parameters[0].descriptor != b"Ljava/lang/Object;"
    {
        return Ok(None);
    }
    Ok(Some(GenericOuterRelationProof {
        enclosing,
        simple_name: simple_name.to_owned(),
        type_parameter_count: 1,
        top_level_non_generic_owner: false,
    }))
}

/// Prove the reciprocal row on `Outer` needed to spell the selected `Outer.A<T>` source path.
/// This first slice accepts one non-generic top-level enclosing class; another member edge needs a
/// separate path proof.
pub(crate) fn enclosing_relation_agrees(
    bytes: &[u8],
    enclosing_class: &ClassMemberFacts,
    generic_outer: &GenericOuterRelationProof,
    budget: &mut Budget,
) -> Result<bool> {
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_PRIVATE: u16 = 0x0002;
    const ACC_PROTECTED: u16 = 0x0004;
    const ACC_STATIC: u16 = 0x0008;
    const ACC_INTERFACE: u16 = 0x0200;
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_ANNOTATION: u16 = 0x2000;
    const ACC_ENUM: u16 = 0x4000;

    budget.poll()?;
    if enclosing_class.stopped_at.is_some()
        || enclosing_class.this_class.raw().0 != generic_outer.enclosing.as_bytes()
        || enclosing_class.access_flags
            & (ACC_PUBLIC | ACC_INTERFACE | ACC_ABSTRACT | ACC_ANNOTATION | ACC_ENUM)
            != ACC_PUBLIC
        || enclosing_class
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Signature")
    {
        return Ok(false);
    }
    let pool = match class_constant_pool(bytes, budget) {
        Ok(pool) => pool,
        Err(error) => return refusal_or_stop(error).map(|_: Option<()>| false),
    };
    let shells: Vec<_> = enclosing_class
        .attributes
        .iter()
        .filter(|attribute| {
            matches!(
                attribute.name.raw().0.as_slice(),
                b"InnerClasses" | b"EnclosingMethod"
            )
        })
        .cloned()
        .collect();
    let nesting = match attribute_facts(bytes, &shells, &pool, budget) {
        Ok(nesting) => nesting,
        Err(error) => return refusal_or_stop(error).map(|_: Option<()>| false),
    };
    if nesting.enclosing_method.is_some()
        || nesting.inner_classes.iter().any(|entry| {
            cp_class_name(&pool, entry.class_index)
                .is_ok_and(|name| name.0 == generic_outer.enclosing.as_bytes())
        })
    {
        return Ok(false);
    }
    if generic_outer.top_level_non_generic_owner {
        let owner_simple_name = generic_outer
            .enclosing
            .rsplit('/')
            .next()
            .unwrap_or_default();
        return Ok(generic_outer.simple_name.is_empty()
            && generic_outer.type_parameter_count == 0
            && !owner_simple_name.is_empty()
            && !owner_simple_name.contains('$')
            && jarde_java::names::is_java_identifier(owner_simple_name));
    }
    let member = format!("{}${}", generic_outer.enclosing, generic_outer.simple_name);
    let rows: Vec<_> = nesting
        .inner_classes
        .iter()
        .filter(|entry| {
            cp_class_name(&pool, entry.class_index).is_ok_and(|name| name.0 == member.as_bytes())
        })
        .collect();
    let [row] = rows.as_slice() else {
        return Ok(false);
    };
    Ok(row.outer_class_index != 0
        && row.access_flags & (ACC_PUBLIC | ACC_PRIVATE | ACC_PROTECTED | ACC_STATIC)
            == (ACC_PUBLIC | ACC_STATIC)
        && cp_class_name(&pool, row.outer_class_index)
            .is_ok_and(|name| name.0 == generic_outer.enclosing.as_bytes())
        && row
            .inner_name
            .as_ref()
            .is_some_and(|name| name.0 == generic_outer.simple_name.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use jarde_reader::budget::{CancellationToken, Limits};
    use jarde_reader::classfile::{CpEntryKind, class_constant_pool, class_member_facts};
    use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
    use std::io::{Cursor, Read, Write};

    const INNER: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/classes/nested/SimpleOuter$Inner.class"
    );
    const OWNER: &str = "nested/SimpleOuter$Inner";
    const DESCRIPTOR: &str = "(Lnested/SimpleOuter;I)V";
    const GENERIC_JAR: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/non-generic-outer-generic-member/original-outer-g-none.jar"
    );
    const GENERIC_OWNER: &str = "minimal/Outer$Inner";
    const GENERIC_DESCRIPTOR: &str = "(Lminimal/Outer;Ljava/lang/Object;)V";
    const FAMILY_JAR: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-26/named-member-family-stage1/fixture.jar"
    );
    const OUTER_SUPER_JAR: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-26/named-member-outer-receiver/variants/fixture.jar"
    );

    fn archive_class_bytes(archive_bytes: &[u8], path: &[u8]) -> Vec<u8> {
        let archive = rawzip::ZipArchive::from_slice(archive_bytes).unwrap();
        let mut entries = archive.entries();
        while let Some(header) = entries.next_entry().unwrap() {
            if header.file_path().as_ref() == path {
                let entry = archive.get_entry(header.wayfinder()).unwrap();
                let decoder = flate2::bufread::DeflateDecoder::new(entry.data());
                let mut reader = entry.verifying_reader(decoder);
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).unwrap();
                return bytes;
            }
        }
        panic!("frozen archive contains requested class")
    }

    fn effectful_bridge_jar() -> Vec<u8> {
        let entries: &[(&[u8], &[u8])] = &[
            (
                b"EffectsBase.class",
                include_bytes!(
                    "../openspec/evidence/java-syntax-2026-09-27/outer-super-bridge-effects/EffectsBase.class"
                ),
            ),
            (
                b"OuterSuperEffects.class",
                include_bytes!(
                    "../openspec/evidence/java-syntax-2026-09-27/outer-super-bridge-effects/OuterSuperEffects.class"
                ),
            ),
            (
                b"OuterSuperEffects$Member.class",
                include_bytes!(
                    "../openspec/evidence/java-syntax-2026-09-27/outer-super-bridge-effects/OuterSuperEffects$Member.class"
                ),
            ),
        ];
        let mut output = Cursor::new(Vec::new());
        {
            let mut archive = ZipArchiveWriter::new(&mut output);
            for (name, bytes) in entries {
                let (mut entry, config) = archive
                    .new_file(EntryPath::verbatim(name.to_vec()))
                    .compression_method(CompressionMethod::new(0))
                    .start()
                    .unwrap();
                let mut writer = config.wrap(&mut entry);
                writer.write_all(bytes).unwrap();
                let (_, descriptor) = writer.finish().unwrap();
                entry.finish(descriptor).unwrap();
            }
            archive.finish().unwrap();
        }
        output.into_inner()
    }

    fn family_bytes(path: &[u8]) -> Vec<u8> {
        let archive = rawzip::ZipArchive::from_slice(FAMILY_JAR).unwrap();
        let mut entries = archive.entries();
        while let Some(header) = entries.next_entry().unwrap() {
            if header.file_path().as_ref() == path {
                let entry = archive.get_entry(header.wayfinder()).unwrap();
                let decoder = flate2::bufread::DeflateDecoder::new(entry.data());
                let mut reader = entry.verifying_reader(decoder);
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).unwrap();
                return bytes;
            }
        }
        panic!("frozen family jar contains requested class")
    }

    fn family_facts(
        bytes: &[u8],
        budget: &mut Budget,
    ) -> (
        ClassMemberFacts,
        Vec<CpEntryFacts>,
        ClassSourceAssemblyContext,
    ) {
        let facts = class_member_facts(bytes, budget).unwrap();
        let pool = class_constant_pool(bytes, budget).unwrap();
        let shells: Vec<_> = facts
            .attributes
            .iter()
            .filter(|attribute| {
                matches!(
                    attribute.name.raw().0.as_slice(),
                    b"InnerClasses" | b"EnclosingMethod"
                )
            })
            .cloned()
            .collect();
        let nesting =
            crate::class_source::read_class_source_assembly_context(bytes, &shells, &pool, budget)
                .unwrap();
        (facts, pool, nesting)
    }

    #[test]
    fn bridge_load_and_return_opcodes_follow_jvm_parameter_categories() {
        for (descriptor, generic, implicit, returns) in [
            (b"I".as_slice(), 0x15, 0x1a, 0xac),
            (b"J", 0x16, 0x1e, 0xad),
            (b"F", 0x17, 0x22, 0xae),
            (b"D", 0x18, 0x26, 0xaf),
            (b"Ljava/lang/Object;", 0x19, 0x2a, 0xb0),
            (b"[I", 0x19, 0x2a, 0xb0),
        ] {
            assert!(bridge_load_opcode(generic, descriptor));
            for opcode in implicit..=implicit + 3 {
                assert!(bridge_load_opcode(opcode, descriptor));
            }
            assert!(bridge_return_opcode(returns, Some(descriptor)));
        }
        assert!(bridge_return_opcode(0xb1, None));
        assert!(!bridge_load_opcode(0x2a, b"I"));
        assert!(!bridge_return_opcode(0xb1, Some(b"I")));
    }

    #[test]
    fn outer_super_bridge_and_call_require_exact_body_target_and_capture_value() {
        let engine = Engine::new();
        let mut budget = budget();
        let snapshot = engine
            .open(ArtifactInput::bytes(OUTER_SUPER_JAR.to_vec()), &mut budget)
            .unwrap();
        let environment = EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        };
        let root = match engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("OuterReceiverCases"),
                    },
                    environment: environment.clone(),
                },
                &mut budget,
            )
            .unwrap()
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("frozen Outer must resolve: {other:?}"),
        };
        let child = match engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("OuterReceiverCases$Member"),
                    },
                    environment: environment.clone(),
                },
                &mut budget,
            )
            .unwrap()
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("frozen Member must resolve: {other:?}"),
        };
        let parent = match engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("ReceiverBase"),
                    },
                    environment: environment.clone(),
                },
                &mut budget,
            )
            .unwrap()
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("frozen direct superclass must resolve: {other:?}"),
        };
        let selected = environment.build(std::slice::from_ref(&snapshot)).unwrap();
        let outer_bytes = archive_class_bytes(OUTER_SUPER_JAR, b"OuterReceiverCases.class");
        let parent_bytes = archive_class_bytes(OUTER_SUPER_JAR, b"ReceiverBase.class");
        let child_bytes = archive_class_bytes(OUTER_SUPER_JAR, b"OuterReceiverCases$Member.class");
        let outer_facts = class_member_facts(&outer_bytes, &mut budget).unwrap();
        let parent_facts = class_member_facts(&parent_bytes, &mut budget).unwrap();
        let child_facts = class_member_facts(&child_bytes, &mut budget).unwrap();
        let bridge_id = root
            .methods
            .iter()
            .find(|method| method.item.identity.name.0 == b"access$101")
            .unwrap()
            .item
            .identity
            .clone();
        let bridge_analysis = jarde_jvm::analyze_method_ir(
            std::slice::from_ref(&snapshot),
            &crate::ir::MethodAnalysisRequest {
                environment: selected.clone(),
                method: bridge_id.clone(),
                stages: MethodOperation::Analysis.stages().to_vec(),
            },
            &mut budget,
        )
        .unwrap();
        let bridge = prove_outer_super_bridge(
            &outer_facts,
            &parent_facts,
            &parent.class,
            &bridge_id,
            bridge_analysis.ir(),
            &mut budget,
        )
        .unwrap()
        .unwrap();
        assert_eq!(bridge.invoke_bci, 1);
        assert_eq!(bridge.target_owner.0, b"ReceiverBase");
        assert_eq!(bridge.target_name.0, b"value");
        assert_eq!(bridge.target_descriptor.0, b"()I");
        assert_eq!(bridge.target_method.owner, parent.class);
        assert_eq!(bridge.target_method.name, bridge.target_name);
        assert_eq!(bridge.target_method.descriptor, bridge.target_descriptor);

        let mut wrong_parent = parent_facts.clone();
        wrong_parent.this_class = outer_facts.this_class.clone();
        assert!(
            prove_outer_super_bridge(
                &outer_facts,
                &wrong_parent,
                &parent.class,
                &bridge_id,
                bridge_analysis.ir(),
                &mut budget,
            )
            .unwrap()
            .is_err()
        );
        let mut wrong_target = parent_facts.clone();
        wrong_target
            .methods
            .iter_mut()
            .find(|method| method.name.raw().0 == b"value")
            .unwrap()
            .access_flags |= 0x0008;
        assert!(
            prove_outer_super_bridge(
                &outer_facts,
                &wrong_target,
                &parent.class,
                &bridge_id,
                bridge_analysis.ir(),
                &mut budget,
            )
            .unwrap()
            .is_err()
        );
        let mut extra_effect = bridge_analysis.ir().code().unwrap().clone();
        extra_effect
            .instructions
            .insert(1, extra_effect.instructions[0].clone());
        assert!(
            prove_outer_super_bridge_body(
                &outer_facts,
                &parent_facts,
                &parent.class,
                &bridge_id,
                bridge_analysis.ir(),
                &extra_effect,
                &mut budget,
            )
            .unwrap()
            .is_err()
        );

        let analyses: Vec<_> = child_facts
            .methods
            .iter()
            .map(|method| {
                let id = PhysicalMethodId {
                    owner: child.class.clone(),
                    name: method.name.raw().clone(),
                    descriptor: method.descriptor.raw().clone(),
                };
                let analysis = jarde_jvm::analyze_method_ir(
                    std::slice::from_ref(&snapshot),
                    &crate::ir::MethodAnalysisRequest {
                        environment: selected.clone(),
                        method: id.clone(),
                        stages: MethodOperation::Analysis.stages().to_vec(),
                    },
                    &mut budget,
                )
                .unwrap();
                (id, analysis)
            })
            .collect();
        let irs: Vec<_> = analyses
            .iter()
            .map(|(id, analysis)| (id.clone(), analysis.ir()))
            .collect();
        let capture = prove_family_capture(b"OuterReceiverCases", &child_facts, &irs, &mut budget)
            .unwrap()
            .unwrap();
        let (caller, compare) = analyses
            .iter()
            .find(|(id, _)| id.name.0 == b"compare")
            .unwrap();
        let call = prove_outer_super_call(caller, compare.ir(), &capture, &bridge, 38, &mut budget)
            .unwrap()
            .unwrap();
        assert_eq!((call.capture_read_bci, call.call_bci), (35, 38));
        assert!(call.argument_bcis.is_empty());

        // `access$000(other)` has the same Outer parameter descriptor, but its SSA value is
        // aload_1 rather than the captured field. A proof-unit symbolic substitution isolates
        // that distinction from the different helper name.
        let mut same_type_other = bridge.clone();
        same_type_other.bridge.name = JvmBytes(b"access$000".to_vec());
        let refusal = prove_outer_super_call(
            caller,
            compare.ir(),
            &capture,
            &same_type_other,
            8,
            &mut budget,
        )
        .unwrap()
        .unwrap_err();
        assert!(refusal.contains("capture"), "{refusal}");
        assert!(
            prove_outer_super_call(caller, compare.ir(), &capture, &bridge, 50, &mut budget)
                .unwrap()
                .is_err()
        ); // Member's own invokevirtual value()
    }

    #[test]
    fn outer_super_call_keeps_effectful_arguments_in_physical_order() {
        let jar = effectful_bridge_jar();
        let engine = Engine::new();
        let mut budget = budget();
        let snapshot = engine
            .open(ArtifactInput::bytes(jar.clone()), &mut budget)
            .unwrap();
        let environment = EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        };
        let selected = environment.build(std::slice::from_ref(&snapshot)).unwrap();
        let report_for = |name: &str, budget: &mut Budget| match engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal(name),
                    },
                    environment: environment.clone(),
                },
                budget,
            )
            .unwrap()
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("effect fixture class must resolve: {other:?}"),
        };
        let root = report_for("OuterSuperEffects", &mut budget);
        let child = report_for("OuterSuperEffects$Member", &mut budget);
        let parent = report_for("EffectsBase", &mut budget);
        let outer_facts = class_member_facts(include_bytes!("../openspec/evidence/java-syntax-2026-09-27/outer-super-bridge-effects/OuterSuperEffects.class"), &mut budget).unwrap();
        let parent_facts = class_member_facts(include_bytes!("../openspec/evidence/java-syntax-2026-09-27/outer-super-bridge-effects/EffectsBase.class"), &mut budget).unwrap();
        let child_facts = class_member_facts(include_bytes!("../openspec/evidence/java-syntax-2026-09-27/outer-super-bridge-effects/OuterSuperEffects$Member.class"), &mut budget).unwrap();
        let bridge_id = root
            .methods
            .iter()
            .find(|method| method.item.identity.name.0 == b"access$001")
            .unwrap()
            .item
            .identity
            .clone();
        let bridge_analysis = jarde_jvm::analyze_method_ir(
            std::slice::from_ref(&snapshot),
            &crate::ir::MethodAnalysisRequest {
                environment: selected.clone(),
                method: bridge_id.clone(),
                stages: MethodOperation::Analysis.stages().to_vec(),
            },
            &mut budget,
        )
        .unwrap();
        let bridge = prove_outer_super_bridge(
            &outer_facts,
            &parent_facts,
            &parent.class,
            &bridge_id,
            bridge_analysis.ir(),
            &mut budget,
        )
        .unwrap()
        .unwrap();
        assert_eq!(bridge.target_descriptor.0, b"(II)I");
        assert_eq!(bridge.invoke_bci, 3);
        let analyses: Vec<_> = child_facts
            .methods
            .iter()
            .map(|method| {
                let id = PhysicalMethodId {
                    owner: child.class.clone(),
                    name: method.name.raw().clone(),
                    descriptor: method.descriptor.raw().clone(),
                };
                let analysis = jarde_jvm::analyze_method_ir(
                    std::slice::from_ref(&snapshot),
                    &crate::ir::MethodAnalysisRequest {
                        environment: selected.clone(),
                        method: id.clone(),
                        stages: MethodOperation::Analysis.stages().to_vec(),
                    },
                    &mut budget,
                )
                .unwrap();
                (id, analysis)
            })
            .collect();
        let irs: Vec<_> = analyses
            .iter()
            .map(|(id, analysis)| (id.clone(), analysis.ir()))
            .collect();
        let capture = prove_family_capture(b"OuterSuperEffects", &child_facts, &irs, &mut budget)
            .unwrap()
            .unwrap();
        let (caller, run) = analyses.iter().find(|(id, _)| id.name.0 == b"run").unwrap();
        let proof = prove_outer_super_call(caller, run.ir(), &capture, &bridge, 14, &mut budget)
            .unwrap()
            .unwrap();
        assert_eq!(proof.capture_read_bci, 1);
        assert_eq!(proof.argument_bcis, vec![6, 11]);
    }

    #[test]
    fn family_capture_distinguishes_other_and_refuses_unclosed_shapes() {
        let engine = Engine::new();
        let mut budget = budget();
        let snapshot = engine
            .open(ArtifactInput::bytes(FAMILY_JAR.to_vec()), &mut budget)
            .unwrap();
        let environment = EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        };
        let report = match engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("NamedMemberFamilyStage1"),
                    },
                    environment: environment.clone(),
                },
                &mut budget,
            )
            .unwrap()
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("unique frozen family: {other:?}"),
        };
        let ClassSourceMemberFamily::Prepared { child, .. } = &report.member_family else {
            panic!("frozen family relation must prepare")
        };
        let child_bytes = family_bytes(b"NamedMemberFamilyStage1$Member.class");
        let (facts, _, _) = family_facts(&child_bytes, &mut budget);
        let selected = environment.build(std::slice::from_ref(&snapshot)).unwrap();
        let analyses: Vec<_> = facts
            .methods
            .iter()
            .filter(|method| {
                method
                    .attributes
                    .iter()
                    .any(|attribute| attribute.name.raw().0 == b"Code")
            })
            .map(|method| {
                let id = PhysicalMethodId {
                    owner: child.class.clone(),
                    name: method.name.raw().clone(),
                    descriptor: method.descriptor.raw().clone(),
                };
                let analysis = jarde_jvm::analyze_method_ir(
                    std::slice::from_ref(&snapshot),
                    &crate::ir::MethodAnalysisRequest {
                        environment: selected.clone(),
                        method: id.clone(),
                        stages: MethodOperation::Analysis.stages().to_vec(),
                    },
                    &mut budget,
                )
                .unwrap();
                (id, analysis)
            })
            .collect();
        let irs: Vec<_> = analyses
            .iter()
            .map(|(id, analysis)| (id.clone(), analysis.ir()))
            .collect();
        let proved = prove_family_capture(b"NamedMemberFamilyStage1", &facts, &irs, &mut budget)
            .unwrap()
            .unwrap();
        assert_eq!(proved.reads.len(), 1);
        assert_eq!(proved.reads[0].bci, 8);
        let (read_id, read_ir) = irs.iter().find(|(id, _)| id.name.0 == b"read").unwrap();
        let ssa = read_ir.ssa().unwrap();
        let other_receiver = ssa_instruction(ssa, 1).unwrap().reads()[0].1;
        let capture_receiver = ssa_instruction(ssa, 8).unwrap().reads()[0].1;
        assert!(!value_from_this_load(ssa, other_receiver));
        assert!(value_from_this_load(ssa, capture_receiver));
        assert!(!capture_write_is_unique_site(
            read_id,
            8,
            &proved.constructor
        ));
        assert_eq!(proved.reads[0].consumer_bcis, vec![11]);

        // Proof-unit mutations keep the real SSA table and change only the access being checked.
        // The first simulates a capture read on explicit `other`; the second an extra field write.
        let mut extra_read = read_ir.code().unwrap().clone();
        extra_read.instructions[1].constant_pool_index = Some(1);
        let refusal = scan_capture_method_uses(
            read_id,
            read_ir,
            &extra_read,
            ssa,
            &proved.constructor,
            b"NamedMemberFamilyStage1$Member",
            b"this$0",
            b"LNamedMemberFamilyStage1;",
            &mut budget,
        )
        .unwrap()
        .unwrap_err();
        assert!(refusal.contains("receiver"), "{refusal}");
        let mut extra_write = read_ir.code().unwrap().clone();
        extra_write.instructions[5].opcode = 0xb5; // BCI 8 in the frozen read method
        let refusal = scan_capture_method_uses(
            read_id,
            read_ir,
            &extra_write,
            ssa,
            &proved.constructor,
            b"NamedMemberFamilyStage1$Member",
            b"this$0",
            b"LNamedMemberFamilyStage1;",
            &mut budget,
        )
        .unwrap()
        .unwrap_err();
        assert!(refusal.contains("extra write"), "{refusal}");

        let mut duplicate_field = facts.clone();
        duplicate_field
            .fields
            .push(duplicate_field.fields[0].clone());
        duplicate_field.field_count += 1;
        assert!(
            prove_family_capture(
                b"NamedMemberFamilyStage1",
                &duplicate_field,
                &irs,
                &mut budget
            )
            .unwrap()
            .is_err()
        );
        let mut nonfinal_field = facts.clone();
        nonfinal_field.fields[0].access_flags &= !0x0010;
        assert!(
            prove_family_capture(
                b"NamedMemberFamilyStage1",
                &nonfinal_field,
                &irs,
                &mut budget
            )
            .unwrap()
            .is_err()
        );
        let mut duplicate_constructor = facts.clone();
        duplicate_constructor
            .methods
            .push(duplicate_constructor.methods[0].clone());
        duplicate_constructor.method_count += 1;
        assert!(
            prove_family_capture(
                b"NamedMemberFamilyStage1",
                &duplicate_constructor,
                &irs,
                &mut budget
            )
            .unwrap()
            .is_err()
        );

        let constructor_ir = irs.iter().find(|(id, _)| id.name.0 == b"<init>").unwrap().1;
        let mut code = constructor_ir.code().unwrap().clone();
        assert!(capture_constructor_shape(&code));
        code.instructions[2].opcode = 0xb7; // proof-unit: a this() delegation in place of the capture write
        assert!(!capture_constructor_shape(&code));
        code.instructions[2].opcode = 0xb5;
        code.exception_handlers
            .push(jarde_reader::classfile::ExceptionHandlerFact {
                ordinal: 0,
                start_bci: 0,
                end_bci: 5,
                handler_bci: 9,
                catch_type_index: None,
            });
        assert!(!capture_constructor_shape(&code));
    }

    #[test]
    fn family_relation_needs_matching_typed_rows_and_uses_member_flags() {
        let root_bytes = family_bytes(b"NamedMemberFamilyStage1.class");
        let child_bytes = family_bytes(b"NamedMemberFamilyStage1$Member.class");
        let mut budget = budget();
        let (_, root_pool, root_nesting) = family_facts(&root_bytes, &mut budget);
        let (child_facts, child_pool, child_nesting) = family_facts(&child_bytes, &mut budget);
        let FamilyRootScan::Candidate(candidate) = scan_family_root(
            b"NamedMemberFamilyStage1",
            &root_nesting,
            &root_pool,
            &mut budget,
        )
        .unwrap() else {
            panic!("root typed row names one child")
        };
        assert_eq!(candidate.simple_name, "Member");
        assert_eq!(candidate.access_flags & FAMILY_VISIBILITY_FLAGS, 0);
        assert_eq!(child_facts.access_flags & 0x0001, 0);
        assert!(
            child_relation_agrees(
                b"NamedMemberFamilyStage1",
                &candidate,
                &child_facts,
                &child_nesting,
                &child_pool,
                &mut budget,
            )
            .unwrap()
        );

        let mut no_root_row = root_nesting.clone();
        no_root_row.inner_classes.clear();
        assert_eq!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &no_root_row,
                &root_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Absent
        );
        let mut duplicate_root = root_nesting.clone();
        duplicate_root
            .inner_classes
            .push(root_nesting.inner_classes[0].clone());
        assert!(matches!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &duplicate_root,
                &root_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Refused(_)
        ));
        let mut static_only = root_nesting.clone();
        static_only.inner_classes[0].access_flags |= 0x0008;
        assert_eq!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &static_only,
                &root_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Absent
        );
        let mut static_sibling = root_nesting.clone();
        let mut static_row = root_nesting.inner_classes[0].clone();
        static_row.access_flags |= 0x0008;
        static_sibling.inner_classes.push(static_row);
        assert!(matches!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &static_sibling,
                &root_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Refused(_)
        ));
        let mut sibling_pool = root_pool.clone();
        let mut sibling_class = sibling_pool
            .iter()
            .find(|entry| entry.index == root_nesting.inner_classes[0].class_index)
            .unwrap()
            .clone();
        sibling_class.index = sibling_pool.iter().map(|entry| entry.index).max().unwrap() + 1;
        let CpEntryKind::Class { name, .. } = &mut sibling_class.kind else {
            unreachable!("InnerClasses class index names a Class entry")
        };
        name.0 = b"NamedMemberFamilyStage1$Static".to_vec();
        let sibling_index = sibling_class.index;
        sibling_pool.push(sibling_class);
        static_sibling.inner_classes.last_mut().unwrap().class_index = sibling_index;
        assert!(matches!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &static_sibling,
                &sibling_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Candidate(found) if found == candidate
        ));
        let mut dollar_pool = root_pool.clone();
        let child_entry = dollar_pool
            .iter_mut()
            .find(|entry| entry.index == root_nesting.inner_classes[0].class_index)
            .unwrap();
        let CpEntryKind::Class { name, .. } = &mut child_entry.kind else {
            unreachable!("InnerClasses class index names a Class entry")
        };
        name.0 = b"NamedMemberFamilyStage1$$A".to_vec();
        let mut dollar_row = root_nesting.clone();
        dollar_row.inner_classes[0].inner_name =
            Some(jarde_reader::model::JvmBytes(b"$A".to_vec()));
        assert!(matches!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &dollar_row,
                &dollar_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Candidate(found) if found.simple_name == "$A"
        ));
        let root_class_index = root_pool
            .iter()
            .find_map(|entry| match &entry.kind {
                CpEntryKind::Class { name, .. } if name.0 == b"NamedMemberFamilyStage1" => {
                    Some(entry.index)
                }
                _ => None,
            })
            .unwrap();
        let mut nested_root = root_nesting.clone();
        let mut self_row = root_nesting.inner_classes[0].clone();
        self_row.class_index = root_class_index;
        nested_root.inner_classes.push(self_row);
        assert!(matches!(
            scan_family_root(
                b"NamedMemberFamilyStage1",
                &nested_root,
                &root_pool,
                &mut budget,
            )
            .unwrap(),
            FamilyRootScan::Refused(_)
        ));

        let mut missing_child_row = child_nesting.clone();
        missing_child_row.inner_classes.clear();
        assert!(
            !child_relation_agrees(
                b"NamedMemberFamilyStage1",
                &candidate,
                &child_facts,
                &missing_child_row,
                &child_pool,
                &mut budget,
            )
            .unwrap()
        );
        let mut wrong_flags = child_nesting.clone();
        wrong_flags.inner_classes[0].access_flags |= 0x0001;
        assert!(
            !child_relation_agrees(
                b"NamedMemberFamilyStage1",
                &candidate,
                &child_facts,
                &wrong_flags,
                &child_pool,
                &mut budget,
            )
            .unwrap()
        );
        let mut local = child_nesting.clone();
        local.enclosing_method = Some(jarde_reader::classfile::EnclosingMethodFacts {
            class_index: 1,
            method_index: 1,
        });
        assert!(
            !child_relation_agrees(
                b"NamedMemberFamilyStage1",
                &candidate,
                &child_facts,
                &local,
                &child_pool,
                &mut budget,
            )
            .unwrap()
        );
    }

    #[test]
    fn family_row_scan_observes_real_budget_and_cancellation() {
        let bytes = family_bytes(b"NamedMemberFamilyStage1.class");
        let mut setup = budget();
        let (_, pool, nesting) = family_facts(&bytes, &mut setup);
        let mut limited = Budget::new(Limits {
            analysis_steps: 0,
            ..Limits::default()
        });
        assert!(matches!(
            scan_family_root(b"NamedMemberFamilyStage1", &nesting, &pool, &mut limited),
            Err(Error::BudgetExceeded { .. })
        ));
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let mut cancelled = Budget::with_cancellation_token(Limits::default(), cancellation);
        assert!(matches!(
            scan_family_root(b"NamedMemberFamilyStage1", &nesting, &pool, &mut cancelled),
            Err(Error::Cancelled { .. })
        ));
    }

    fn budget() -> Budget {
        Budget::new(Limits {
            input_bytes: u64::MAX,
            archive_entries: u64::MAX,
            entry_bytes: u64::MAX,
            read_bytes: u64::MAX,
            class_bytes: u64::MAX,
            attribute_bytes: u64::MAX,
            code_bytes: u64::MAX,
            result_items: u64::MAX,
            output_bytes: u64::MAX,
            class_headers: u64::MAX,
            method_bodies: u64::MAX,
            ir_items: u64::MAX,
            ir_edges: u64::MAX,
            analysis_steps: u64::MAX,
            normalization_clones: u64::MAX,
            nested_depth: u64::MAX,
            dependency_depth: u64::MAX,
            elapsed_millis: u64::MAX,
        })
    }

    fn generic_inner_bytes() -> Vec<u8> {
        let archive = rawzip::ZipArchive::from_slice(GENERIC_JAR).expect("frozen jar parses");
        let mut entries = archive.entries();
        while let Some(header) = entries.next_entry().expect("frozen jar directory parses") {
            if header.file_path().as_ref() == b"minimal/Outer$Inner.class" {
                let entry = archive
                    .get_entry(header.wayfinder())
                    .expect("frozen inner entry parses");
                let decoder = flate2::bufread::DeflateDecoder::new(entry.data());
                let mut reader = entry.verifying_reader(decoder);
                let mut bytes = Vec::new();
                reader
                    .read_to_end(&mut bytes)
                    .expect("frozen inner entry inflates and verifies");
                return bytes;
            }
        }
        panic!("frozen jar contains the selected Inner class")
    }

    fn replace_utf8(bytes: &[u8], old: &[u8], new: &[u8]) -> Vec<u8> {
        let mut unlimited = budget();
        let pool = class_constant_pool(bytes, &mut unlimited).expect("constant pool reads");
        let entry = pool
            .iter()
            .find(|entry| matches!(&entry.kind, CpEntryKind::Utf8 { bytes } if bytes.0 == old))
            .expect("fixture contains signature text");
        let start = usize::try_from(entry.span.start).expect("span start fits usize");
        assert_eq!(bytes.get(start), Some(&1), "entry is CONSTANT_Utf8");
        let length_start = start + 1;
        let payload_start = start + 3;
        let payload_end = payload_start + old.len();
        let mut changed = bytes.to_vec();
        changed.splice(payload_start..payload_end, new.iter().copied());
        changed[length_start..payload_start]
            .copy_from_slice(&u16::try_from(new.len()).unwrap().to_be_bytes());
        changed
    }

    #[test]
    fn exact_target_relation_field_and_prologue_are_required() {
        let mut budget = budget();
        let facts = class_member_facts(INNER, &mut budget).unwrap();
        let proof = prove_target(INNER, &facts, OWNER, DESCRIPTOR, &mut budget)
            .unwrap()
            .expect("frozen Java 8 member target proves");
        assert_eq!(proof.outer, "nested/SimpleOuter");
        assert_eq!(proof.simple_name, "Inner");
        assert_eq!(proof.capture_field, "this$0");

        assert!(
            prove_target(INNER, &facts, OWNER, "(Lnested/SimpleOuter;)V", &mut budget)
                .unwrap()
                .is_none()
        );
        let mut no_capture = facts.clone();
        no_capture
            .fields
            .iter_mut()
            .find(|field| field.name.raw().0 == b"this$0")
            .unwrap()
            .access_flags &= !0x1000;
        assert!(
            prove_target(INNER, &no_capture, OWNER, DESCRIPTOR, &mut budget)
                .unwrap()
                .is_none()
        );
        let mut ambiguous_capture = facts.clone();
        let capture = ambiguous_capture
            .fields
            .iter()
            .find(|field| field.name.raw().0 == b"this$0")
            .unwrap()
            .clone();
        ambiguous_capture.fields.push(capture);
        assert!(
            prove_target(INNER, &ambiguous_capture, OWNER, DESCRIPTOR, &mut budget)
                .unwrap()
                .is_none()
        );

        let mut wrong_prologue = INNER.to_vec();
        let constructor = facts
            .methods
            .iter()
            .find(|method| method.name.raw().0 == b"<init>")
            .unwrap();
        let code = method_code_facts(INNER, constructor, &mut budget).unwrap();
        let first_parameter_load = usize::try_from(code.code_span.start).unwrap() + 1;
        wrong_prologue[first_parameter_load] = 0x2c; // aload_2, not the physical outer parameter
        assert!(
            prove_target(&wrong_prologue, &facts, OWNER, DESCRIPTOR, &mut budget)
                .unwrap()
                .is_none()
        );

        let mut wrong_relation = INNER.to_vec();
        let inner_shell = facts
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"InnerClasses")
            .unwrap();
        let outer_index = usize::try_from(inner_shell.content_span.start).unwrap() + 4;
        wrong_relation[outer_index..outer_index + 2].copy_from_slice(&0_u16.to_be_bytes());
        let wrong_facts = class_member_facts(&wrong_relation, &mut budget).unwrap();
        assert!(
            prove_target(
                &wrong_relation,
                &wrong_facts,
                OWNER,
                DESCRIPTOR,
                &mut budget
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn generic_member_signature_and_constructor_tail_are_proved() {
        let bytes = generic_inner_bytes();
        let mut budget = budget();
        let facts = class_member_facts(&bytes, &mut budget).unwrap();
        let proof = prove_target(
            &bytes,
            &facts,
            GENERIC_OWNER,
            GENERIC_DESCRIPTOR,
            &mut budget,
        )
        .unwrap()
        .expect("frozen generic member target proves");
        assert!(proof.generic_diamond);
        assert_eq!(proof.outer, "minimal/Outer");
        assert_eq!(proof.simple_name, "Inner");

        let wrong_class_variable = replace_utf8(
            &bytes,
            b"<V:Ljava/lang/Object;>Ljava/lang/Object;",
            b"<T:Ljava/lang/Object;>Ljava/lang/Object;",
        );
        let wrong_class_facts = class_member_facts(&wrong_class_variable, &mut budget).unwrap();
        assert!(
            prove_target(
                &wrong_class_variable,
                &wrong_class_facts,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );

        for wrong_class_signature in [
            b"<V:Ljava/lang/Number;>Ljava/lang/Object;".as_slice(),
            b"<V:TT;>Ljava/lang/Object;".as_slice(),
        ] {
            let malformed_scope = replace_utf8(
                &bytes,
                b"<V:Ljava/lang/Object;>Ljava/lang/Object;",
                wrong_class_signature,
            );
            let malformed_facts = class_member_facts(&malformed_scope, &mut budget).unwrap();
            assert!(
                prove_target(
                    &malformed_scope,
                    &malformed_facts,
                    GENERIC_OWNER,
                    GENERIC_DESCRIPTOR,
                    &mut budget,
                )
                .unwrap()
                .is_none()
            );
        }

        let wrong_constructor_erasure = replace_utf8(&bytes, b"(TV;)V", b"(Ljava/lang/String;)V");
        let wrong_constructor_facts =
            class_member_facts(&wrong_constructor_erasure, &mut budget).unwrap();
        assert!(
            prove_target(
                &wrong_constructor_erasure,
                &wrong_constructor_facts,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );

        let mut missing_class_signature = facts.clone();
        missing_class_signature
            .attributes
            .retain(|attribute| attribute.name.raw().0 != b"Signature");
        assert!(
            prove_target(
                &bytes,
                &missing_class_signature,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );

        let mut duplicate_class_signature = facts.clone();
        let signature_shell = duplicate_class_signature
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"Signature")
            .unwrap()
            .clone();
        duplicate_class_signature.attributes.push(signature_shell);
        assert!(
            prove_target(
                &bytes,
                &duplicate_class_signature,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );

        let constructor_index = facts
            .methods
            .iter()
            .position(|method| method.name.raw().0 == b"<init>")
            .unwrap();
        let mut missing_constructor_signature = facts.clone();
        missing_constructor_signature.methods[constructor_index]
            .attributes
            .retain(|attribute| attribute.name.raw().0 != b"Signature");
        assert!(
            prove_target(
                &bytes,
                &missing_constructor_signature,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );

        let mut duplicate_constructor_signature = facts.clone();
        let constructor_signature_shell = duplicate_constructor_signature.methods
            [constructor_index]
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"Signature")
            .unwrap()
            .clone();
        duplicate_constructor_signature.methods[constructor_index]
            .attributes
            .push(constructor_signature_shell);
        assert!(
            prove_target(
                &bytes,
                &duplicate_constructor_signature,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );

        let mut overloaded = facts.clone();
        overloaded
            .methods
            .push(overloaded.methods[constructor_index].clone());
        assert!(
            prove_target(
                &bytes,
                &overloaded,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut budget,
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn generic_signature_proof_preserves_budget_and_cancellation_stops() {
        let bytes = generic_inner_bytes();
        let mut setup_budget = budget();
        let facts = class_member_facts(&bytes, &mut setup_budget).unwrap();

        let mut limited = Budget::new(Limits {
            class_bytes: 0,
            ..Limits::default()
        });
        assert!(matches!(
            prove_target(
                &bytes,
                &facts,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut limited,
            ),
            Err(Error::BudgetExceeded { .. })
        ));

        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let mut cancelled = Budget::with_cancellation_token(
            Limits {
                class_bytes: u64::MAX,
                ..Limits::default()
            },
            cancellation,
        );
        assert!(matches!(
            prove_target(
                &bytes,
                &facts,
                GENERIC_OWNER,
                GENERIC_DESCRIPTOR,
                &mut cancelled,
            ),
            Err(Error::Cancelled { .. })
        ));

        let mut plain_budget = budget();
        let plain_facts = class_member_facts(INNER, &mut plain_budget).unwrap();
        let plain = prove_target(INNER, &plain_facts, OWNER, DESCRIPTOR, &mut plain_budget)
            .unwrap()
            .expect("existing non-generic member proof remains available");
        assert!(!plain.generic_diamond);
    }
}
