//! Final, atomic projection of one independently proved nonthrowing handler.
use super::*;
use crate::normal_flow::NormalFlowView;
use crate::region::Recovered;

/// The ordinary walk and Builder keep their original fallback until every check below succeeds.
/// The handler remains a separate certificate owner, never a fabricated normal-flow Region.
pub(crate) fn project_dead_loop_transfer_handler(
    recovered: &mut Recovered,
    program: &mut Program,
    proof: &guard::DeadLoopTransferHandler,
    view: &NormalFlowView,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    if recovered.regions.len() != 4 || program.stmts.len() > 16 || program.stmts.is_empty() {
        return Ok(false);
    }
    // The candidate has 35 physical operations and a fixed shallow normal tree. Pay for the
    // bounded region/statement/source-closure inspection before reading it or growing its sets.
    charge(budget, CountedBudgetDimension::AnalysisSteps, 512, Some(45))?;
    if !normal_tree(recovered, proof) {
        return Ok(false);
    }
    let Some(handler_node) = view.index_of(&proof.handler) else {
        return Ok(false);
    };
    let mut normal_owners = BTreeSet::new();
    for region in &recovered.regions[..3] {
        for block in region.blocks() {
            let Some(node) = view.index_of(&block) else {
                return Ok(false);
            };
            if !normal_owners.insert(node) {
                return Ok(false);
            }
        }
    }
    if recovered.blocks != 8
        || normal_owners.len() != 7
        || normal_owners != recovered.claimed
        || normal_owners.contains(&handler_node)
    {
        return Ok(false);
    }
    let last = program.stmts.len() - 1;
    let quote = &program.stmts[last];
    if !matches!(&quote.kind, StmtKind::Fallback { bcis, .. } if bcis == &proof.elided_bcis())
        || quote.origin.bcis() != BTreeSet::from(proof.elided_bcis())
        || !program.ragged
    {
        return Ok(false);
    }
    let normal = &program.stmts[..last];
    let mut for_index = None;
    for (index, statement) in normal.iter().enumerate() {
        match &statement.kind {
            StmtKind::For {
                init,
                cond,
                update,
                body,
                ..
            } => {
                if for_index.replace(index).is_some()
                    || statement.origin.primary().bci() != 13
                    || init.origin.primary().bci() != 8
                    || update.origin.primary().bci() != 53
                    || !cond.origin.bcis().contains(&13)
                    || body.len() > 4
                    || !flat(init)
                    || !flat(update)
                {
                    return Ok(false);
                }
                let mut branch_count = 0;
                for child in body {
                    if let StmtKind::If {
                        then_body,
                        else_body,
                        ..
                    } = &child.kind
                    {
                        branch_count += 1;
                        if child.origin.primary().bci() != 25
                            || then_body.len() > 3
                            || else_body.len() > 4
                            || !then_body.iter().chain(else_body).all(flat)
                            || then_body
                                .iter()
                                .filter(|s| matches!(s.kind, StmtKind::Break { .. }))
                                .count()
                                != usize::from(proof.transfer_target == 59)
                            || else_body.iter().any(|s| {
                                matches!(s.kind, StmtKind::Break { .. } | StmtKind::Continue { .. })
                            })
                        {
                            return Ok(false);
                        }
                    } else if !flat(child) {
                        return Ok(false);
                    }
                }
                if branch_count != 1 {
                    return Ok(false);
                }
            }
            StmtKind::Declare { .. } | StmtKind::Assign { .. } | StmtKind::Return { .. } => {}
            _ => return Ok(false),
        }
    }
    let Some(for_index) = for_index else {
        return Ok(false);
    };
    if contains_fallback(normal) {
        return Ok(false);
    }
    let mut bcis = Vec::new();
    let mut names = Vec::new();
    for statement in normal {
        stated_by_statement(statement, &mut names, &mut bcis);
    }
    let actual: BTreeSet<_> = bcis.into_iter().collect();
    let expected: BTreeSet<_> = guard::LOOP_TRANSFER_BCIS.into_iter().collect();
    let elided = BTreeSet::from(proof.elided_bcis());
    let required: BTreeSet<_> = expected
        .difference(&elided)
        .copied()
        .filter(|at| ![42, 56].contains(at))
        .collect();
    if !required.is_subset(&actual) || !actual.is_subset(&expected) || !actual.is_disjoint(&elided)
    {
        return Ok(false);
    }
    // 42 -> 53 and 56 -> 10 are the certificate's actual normal transfers. Their control effect
    // is carried by this existing for statement; the handler's four instructions are unreachable.
    let origin = [42, 56, 45, 47, 50, 52]
        .into_iter()
        .fold(program.stmts[for_index].origin.clone(), |origin, bci| {
            origin.plus_derived(Origin::derived(bci))
        });
    let Some(statements) = program.statements.checked_sub(1) else {
        return Ok(false);
    };
    let ragged = contains_fallback(normal);
    normal_owners.insert(handler_node);
    charge(budget, CountedBudgetDimension::IrItems, 6, Some(45))?;
    poll(budget, Some(45))?;
    // Nothing below can refuse: both original values survive every earlier mismatch or Stop.
    program.stmts[for_index].origin = origin;
    program.stmts.pop();
    program.statements = statements;
    program.ragged = ragged;
    recovered.regions.pop();
    recovered.claimed = normal_owners;
    Ok(true)
}

fn flat(statement: &Stmt) -> bool {
    matches!(
        statement.kind,
        StmtKind::Declare { .. }
            | StmtKind::Assign { .. }
            | StmtKind::Break { .. }
            | StmtKind::Continue { .. }
    )
}

fn blocks_are(blocks: &[CanonicalBlockId], expected: &[u32]) -> bool {
    blocks.len() == expected.len()
        && blocks
            .iter()
            .zip(expected)
            .all(|(block, at)| block.bci() == *at && block.path().is_empty())
}
fn straight(region: &Region, expected: &[u32]) -> bool {
    matches!(region, Region::Straight { blocks } if blocks_are(blocks, expected))
}
fn normal_tree(recovered: &Recovered, proof: &guard::DeadLoopTransferHandler) -> bool {
    let [lead, loop_region, tail, fallback] = recovered.regions.as_slice() else {
        return false;
    };
    if !straight(lead, &[0])
        || !straight(tail, &[59])
        || !matches!(fallback, Region::Fallback { blocks, reason: FallbackReason::UncoveredBlocks { blocks: missing } } if blocks == &[proof.handler.clone()] && missing == &[45])
    {
        return false;
    }
    let Region::Loop {
        header,
        tests,
        test_operator: None,
        form: LoopForm::While,
        for_header: Some(counted),
        body,
        exit: Some(exit),
        gateway_origins,
    } = loop_region
    else {
        return false;
    };
    if header != &proof.header
        || tests != &[(proof.header.clone(), 13, Continuation::FallThrough)]
        || counted.init_bci != 8
        || counted.update_bci != 53
        || counted.slot != 4
        || !blocks_are(std::slice::from_ref(&counted.update_block), &[53])
        || !blocks_are(std::slice::from_ref(exit), &[59])
    {
        return false;
    }
    let is_continue = proof.transfer_target == 53;
    let branch = if is_continue {
        let [branch, update] = body.as_slice() else {
            return false;
        };
        if !straight(update, &[53]) || gateway_origins != &[56] {
            return false;
        }
        branch
    } else {
        let [branch] = body.as_slice() else {
            return false;
        };
        if proof.transfer_target != 59 || !gateway_origins.is_empty() {
            return false;
        }
        branch
    };
    let Region::If {
        prefix,
        branch,
        branch_bci: 25,
        then_arm,
        else_arm,
        join: Some(join),
    } = branch
    else {
        return false;
    };
    if !prefix.is_empty()
        || !blocks_are(std::slice::from_ref(branch), &[16])
        || !blocks_are(
            std::slice::from_ref(join),
            &[if is_continue { 53 } else { 59 }],
        )
    {
        return false;
    }
    if is_continue {
        straight(then_arm, &[28]) && straight(else_arm, &[34])
    } else {
        let Region::Sequence { regions } = then_arm.as_ref() else {
            return false;
        };
        let [cleanup, transfer] = regions.as_slice() else {
            return false;
        };
        straight(cleanup, &[28])
            && straight(else_arm, &[34, 53])
            && matches!(transfer, Region::LoopBreak { source_bci: 31, loop_header } if loop_header == &proof.header)
    }
}
