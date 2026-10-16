//! A finite certificate for the two loop transfers whose catch-all has no throwable member.
//! Canonical interval edges and their synthetic caught values remain untouched.
use super::*;
use crate::facts::{ArithmeticOp, ConstantValue};
use jarde_jvm::method_ir::{RefType, Value};

pub(crate) const BCIS: [u32; 35] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 13, 16, 17, 19, 20, 22, 24, 25, 28, 31, 34, 35, 37, 38, 39,
    42, 45, 47, 50, 52, 53, 56, 59, 60,
];
const OPCODES: [u8; 35] = [
    0x03, 0x3c, 0x2a, 0x4d, 0x2c, 0xbe, 0x3e, 0x03, 0x36, 0x15, 0x1d, 0xa2, 0x2c, 0x15, 0x2e, 0x36,
    0x15, 0x06, 0xa0, 0x84, 0xa7, 0x1b, 0x15, 0x60, 0x3c, 0x84, 0xa7, 0x3a, 0x84, 0x19, 0xbf, 0x84,
    0xa7, 0x1b, 0xac,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeadLoopTransferHandler {
    pub(crate) handler: CanonicalBlockId,
    pub(crate) header: CanonicalBlockId,
    pub(crate) transfer_target: u32,
}
impl DeadLoopTransferHandler {
    pub(crate) fn elided_bcis(&self) -> [u32; 4] {
        [45, 47, 50, 52]
    }
}

/// Exact physical form first; subsequently every scan and consumer is paid before inspection.
/// This certificate does not replace either normal cleanup or claim a live catch clause.
pub(super) fn prove(facts: &mut Facts<'_>) -> Result<Option<DeadLoopTransferHandler>, StopReason> {
    if facts.order.as_slice() != BCIS
        || facts.handlers.len() != 3
        || facts.canonical.blocks().len() != 8
        || facts.ssa.blocks().len() != 8
        || facts.ssa.phis().len() > 32
        || facts.ssa.values().len() > 96
        || !facts.canonical.completeness().is_complete()
        || !facts.canonical.unreachable().is_empty()
        || BCIS.iter().zip(OPCODES).any(|(at, opcode)| {
            facts
                .step(*at)
                .is_none_or(|s| s.instruction.opcode() != opcode)
        })
    {
        return Ok(None);
    }
    for at in BCIS {
        facts.charge(at)?;
    }
    for phi in facts.ssa.phis() {
        facts.charge(phi.block().bci())?;
        for _ in phi.inputs() {
            facts.charge(phi.block().bci())?;
        }
    }
    for (_, value) in facts.ssa.values_with_ids() {
        facts.charge(0)?;
        for usage in value.uses() {
            facts.charge(usage.bci().unwrap_or(usage.block().bci()))?;
        }
    }
    if facts.ops.iter().map(|(at, _)| *at).collect::<Vec<_>>() != BCIS {
        return Ok(None);
    }
    for (row, expected) in facts
        .handlers
        .iter()
        .zip([(0, 22, 28), (1, 34, 39), (2, 45, 47)])
    {
        facts.charge(row.start_bci)?;
        if (row.ordinal, row.start_bci, row.end_bci) != expected
            || row.handler_bci != 45
            || row.catch_type_index.is_some()
        {
            return Ok(None);
        }
    }
    let blocks: [(u32, u32, &[u32]); 8] = [
        (0, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8]),
        (10, 16, &[10, 12, 13]),
        (16, 28, &[16, 17, 19, 20, 22, 24, 25]),
        (28, 34, &[28, 31]),
        (34, 45, &[34, 35, 37, 38, 39, 42]),
        (45, 53, &[45, 47, 50, 52]),
        (53, 59, &[53, 56]),
        (59, 61, &[59, 60]),
    ];
    for (at, end, bcis) in blocks {
        facts.charge(at)?;
        let Some(block) = facts.starts.get(&at).copied() else {
            return Ok(None);
        };
        if !block.path().is_empty()
            || facts.end_of(block) != end
            || facts
                .in_block(block)
                .iter()
                .map(SsaInstruction::bci)
                .collect::<Vec<_>>()
                != bcis
        {
            return Ok(None);
        }
    }
    let operations = [
        (0, Operation::Push(ConstantValue::Int(0))),
        (1, Operation::Store { slot: 1 }),
        (2, Operation::Load { slot: 0 }),
        (3, Operation::Store { slot: 2 }),
        (4, Operation::Load { slot: 2 }),
        (5, Operation::ArrayLength),
        (6, Operation::Store { slot: 3 }),
        (7, Operation::Push(ConstantValue::Int(0))),
        (8, Operation::Store { slot: 4 }),
        (10, Operation::Load { slot: 4 }),
        (12, Operation::Load { slot: 3 }),
        (
            13,
            Operation::Comparison {
                op: CompareOp::JumpIfGreaterOrEqual,
                target: 59,
            },
        ),
        (16, Operation::Load { slot: 2 }),
        (17, Operation::Load { slot: 4 }),
        (19, Operation::ArrayLoad),
        (20, Operation::Store { slot: 5 }),
        (22, Operation::Load { slot: 5 }),
        (24, Operation::Push(ConstantValue::Int(3))),
        (
            25,
            Operation::Comparison {
                op: CompareOp::JumpIfDifferent,
                target: 34,
            },
        ),
        (
            28,
            Operation::Increment {
                slot: 1,
                amount: 100,
            },
        ),
        (31, Operation::Transfer),
        (34, Operation::Load { slot: 1 }),
        (35, Operation::Load { slot: 5 }),
        (
            37,
            Operation::Arithmetic {
                op: ArithmeticOp::Add,
            },
        ),
        (38, Operation::Store { slot: 1 }),
        (
            39,
            Operation::Increment {
                slot: 1,
                amount: 100,
            },
        ),
        (42, Operation::Transfer),
        (45, Operation::Store { slot: 6 }),
        (
            47,
            Operation::Increment {
                slot: 1,
                amount: 100,
            },
        ),
        (50, Operation::Load { slot: 6 }),
        (52, Operation::Throw),
        (53, Operation::Increment { slot: 4, amount: 1 }),
        (56, Operation::Transfer),
        (59, Operation::Load { slot: 1 }),
        (60, Operation::Return),
    ];
    for (at, op) in &operations {
        facts.charge(*at)?;
        if facts.op(*at) != Some(op) {
            return Ok(None);
        }
    }
    let mut edges = Vec::new();
    for edge in facts.canonical.edges() {
        facts.charge(edge.from().bci())?;
        let kind = match edge.kind() {
            CanonicalEdgeKind::Normal => None,
            CanonicalEdgeKind::Exception { handler_ordinal } => Some(handler_ordinal),
            _ => return Ok(None),
        };
        if !edge.from().path().is_empty() || !edge.to().path().is_empty() {
            return Ok(None);
        }
        edges.push((edge.from().bci(), edge.to().bci(), kind));
    }
    let targets: Vec<_> = edges
        .iter()
        .filter(|(from, _, kind)| *from == 28 && kind.is_none())
        .map(|(_, to, _)| *to)
        .collect();
    let [target] = targets.as_slice() else {
        return Ok(None);
    };
    if ![53, 59].contains(target) {
        return Ok(None);
    }
    let mut expected = vec![
        (0, 10, None),
        (10, 16, None),
        (10, 59, None),
        (16, 28, None),
        (16, 34, None),
        (16, 45, Some(0)),
        (28, *target, None),
        (34, 53, None),
        (34, 45, Some(1)),
        (45, 45, Some(2)),
        (53, 10, None),
    ];
    edges.sort_unstable();
    expected.sort_unstable();
    if edges != expected {
        return Ok(None);
    }
    if facts.canonical.handler_rows().len() != 3 {
        return Ok(None);
    }
    for (row, (ordinal, protected)) in
        facts
            .canonical
            .handler_rows()
            .iter()
            .zip([(0, 16), (1, 34), (2, 45)])
    {
        facts.charge(protected)?;
        if row.ordinal() != ordinal
            || row.handler_bci() != 45
            || row.catch_type_index().is_some()
            || row.handler() != Some(facts.starts[&45])
            || row.protected() != [facts.starts[&protected].clone()]
        {
            return Ok(None);
        }
    }
    let Some(header_node) = facts.view.index_of(facts.starts[&10]) else {
        return Ok(None);
    };
    let Some(handler_node) = facts.view.index_of(facts.starts[&45]) else {
        return Ok(None);
    };
    let Some(latch) = facts.view.index_of(facts.starts[&53]) else {
        return Ok(None);
    };
    let Some(natural) = facts.view.loop_entered_at(header_node) else {
        return Ok(None);
    };
    let members = if *target == 53 {
        &[10, 16, 28, 34, 53][..]
    } else {
        &[10, 16, 34, 53][..]
    };
    let expected_nodes: Option<BTreeSet<_>> = members
        .iter()
        .map(|at| facts.view.index_of(facts.starts[at]))
        .collect();
    if natural.is_irreducible()
        || Some(natural.blocks()) != expected_nodes.as_ref()
        || natural.latches() != &BTreeSet::from([latch])
        || !facts.view.predecessors(handler_node).is_empty()
        || !facts.view.successors(handler_node).is_empty()
    {
        return Ok(None);
    }
    if facts.ssa.effects().instructions().len() != 35 {
        return Ok(None);
    }
    for at in BCIS {
        facts.charge(at)?;
        let matching: Vec<_> = facts
            .ssa
            .effects()
            .instructions()
            .iter()
            .filter(|e| e.bci() == at)
            .collect();
        let [effect] = matching.as_slice() else {
            return Ok(None);
        };
        let covering = if (22..28).contains(&at) {
            &[0][..]
        } else if (34..39).contains(&at) {
            &[1][..]
        } else if at == 45 {
            &[2][..]
        } else {
            &[][..]
        };
        if facts
            .covering(at)
            .iter()
            .map(|r| r.ordinal)
            .collect::<Vec<_>>()
            != covering
            || effect.may_throw() != [5, 19, 52].contains(&at)
            || !effect.handlers().is_empty()
        {
            return Ok(None);
        }
    }
    // These are the actual eligible instruction sites, not the three conservative block edges.
    if facts.canonical.throw_sites().len() != 3 {
        return Ok(None);
    }
    let mut throw_sites = Vec::new();
    for site in facts.canonical.throw_sites() {
        facts.charge(site.bci())?;
        if !site.handlers().is_empty() || !site.block().path().is_empty() {
            return Ok(None);
        }
        throw_sites.push((site.bci(), site.opcode(), site.block().bci()));
    }
    throw_sites.sort_unstable();
    if throw_sites != [(5, 0xbe, 0), (19, 0x2e, 16), (52, 0xbf, 45)] {
        return Ok(None);
    }
    let Some(caught) = facts
        .ssa
        .phis()
        .iter()
        .find(|p| p.block() == facts.starts[&45] && p.slot() == Slot::Stack(0))
    else {
        return Ok(None);
    };
    if caught.inputs().len() != 3
        || facts.ssa.value(caught.value()).replaced_by().is_some()
        || !matches!(
            facts.ssa.value(caught.value()).ty(),
            Value::Ref(RefType::Unknown)
        )
    {
        return Ok(None);
    }
    if !matches!(facts.ssa.value(caught.value()).def(),Definition::Phi{block,slot:Slot::Stack(0)} if block==facts.starts[&45])
    {
        return Ok(None);
    }
    for (input, at) in caught.inputs().iter().zip([16, 34, 45]) {
        facts.charge(at)?;
        let PhiInput::Value(value) = input else {
            return Ok(None);
        };
        let v = facts.ssa.value(*value);
        if !matches!(v.def(),Definition::Caught{block,bci} if block==facts.starts[&at]&&*bci==at)
            || !matches!(v.ty(), Value::Ref(RefType::Unknown))
            || v.replaced_by().is_some()
            || !uses_are(facts, *value, &[(45, None)])
        {
            return Ok(None);
        }
    }
    let (Some(save), Some(increment), Some(load), Some(throw)) = (
        facts.step(45),
        facts.step(47),
        facts.step(50),
        facts.step(52),
    ) else {
        return Ok(None);
    };
    if save.instruction.reads() != [(Slot::Stack(0), caught.value())]
        || save.instruction.writes().len() != 1
        || load.instruction.reads().len() != 1
        || load.instruction.writes().len() != 1
        || increment.instruction.reads().len() != 1
        || increment.instruction.writes().len() != 1
        || !throw.instruction.writes().is_empty()
    {
        return Ok(None);
    }
    let saved = save.instruction.writes()[0].1;
    let loaded = load.instruction.writes()[0].1;
    let updated = increment.instruction.writes()[0].1;
    if save.instruction.writes()[0].0 != Slot::Local(6)
        || load.instruction.reads() != [(Slot::Local(6), saved)]
        || load.instruction.writes()[0].0 != Slot::Stack(0)
        || throw.instruction.reads() != [(Slot::Stack(0), loaded)]
        || !uses_are(facts, caught.value(), &[(45, Some(45))])
        || !uses_are(facts, saved, &[(45, Some(50))])
        || !uses_are(facts, loaded, &[(45, Some(52))])
        || !uses_are(facts, updated, &[(45, None)])
    {
        return Ok(None);
    }
    for (id, at, reference) in [(saved, 45, true), (loaded, 50, true), (updated, 47, false)] {
        facts.charge(at)?;
        let value = facts.ssa.value(id);
        if !matches!(value.def(),Definition::Instruction{block,bci} if block==facts.starts[&45]&&*bci==at)
            || value.replaced_by().is_some()
            || if reference {
                !matches!(value.ty(), Value::Ref(RefType::Unknown))
            } else {
                value.ty() != &Value::Int
            }
        {
            return Ok(None);
        }
    }
    let Some(c) = facts
        .ssa
        .phis()
        .iter()
        .find(|p| p.block() == facts.starts[&45] && p.slot() == Slot::Local(1))
    else {
        return Ok(None);
    };
    let Some(header_c) = facts
        .ssa
        .phis()
        .iter()
        .find(|p| p.block() == facts.starts[&10] && p.slot() == Slot::Local(1))
    else {
        return Ok(None);
    };
    let Some(normal) = facts
        .step(39)
        .and_then(|s| s.instruction.writes().first())
        .map(|(_, v)| *v)
    else {
        return Ok(None);
    };
    if c.inputs()
        != [
            PhiInput::Value(header_c.value()),
            PhiInput::Value(normal),
            PhiInput::Value(updated),
        ]
        || facts.ssa.value(c.value()).ty() != &Value::Int
        || facts.ssa.value(c.value()).replaced_by().is_some()
        || increment.instruction.reads() != [(Slot::Local(1), c.value())]
        || increment.instruction.writes()[0].0 != Slot::Local(1)
        || !uses_are(facts, c.value(), &[(45, Some(47))])
    {
        return Ok(None);
    }
    // The two ordinary copies remain in the existing normal AST; each still writes its own value.
    for at in [28, 39] {
        facts.charge(at)?;
        let s = facts.step(at).unwrap();
        if s.instruction.reads().len() != 1
            || s.instruction.writes().len() != 1
            || s.instruction.reads()[0].0 != Slot::Local(1)
            || s.instruction.writes()[0].0 != Slot::Local(1)
            || facts.ssa.value(s.instruction.reads()[0].1).ty() != &Value::Int
            || facts.ssa.value(s.instruction.writes()[0].1).ty() != &Value::Int
        {
            return Ok(None);
        }
    }
    facts.charge(60)?;
    poll(facts.budget, Some(60))?;
    Ok(Some(DeadLoopTransferHandler {
        handler: facts.starts[&45].clone(),
        header: facts.starts[&10].clone(),
        transfer_target: *target,
    }))
}

fn uses_are(facts: &Facts<'_>, value: ValueId, expected: &[(u32, Option<u32>)]) -> bool {
    let mut actual: Vec<_> = facts
        .ssa
        .value(value)
        .uses()
        .iter()
        .map(|u| (u.block().bci(), u.bci()))
        .collect();
    let mut expected = expected.to_vec();
    actual.sort_unstable();
    expected.sort_unstable();
    actual == expected
        && facts
            .ssa
            .value(value)
            .uses()
            .iter()
            .all(|u| u.block().path().is_empty())
}
