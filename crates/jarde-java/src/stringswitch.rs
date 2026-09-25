//! Bounded, same-method proof of javac's two-switch String lowering.
//!
//! This is only a certificate for the region/build step. It neither changes a region nor removes
//! an instruction. A refusal leaves the existing two integer switches untouched.
use std::collections::{BTreeMap, BTreeSet};

use jarde_jvm::method_ir::{Definition, MethodIr, PhiInput, Slot};
use jarde_reader::budget::{Budget, CountedBudgetDimension};

use crate::decode::Operations;
use crate::facts::{CompareOp, ConstantValue, InvokeKind, Operation};
use crate::stop::{StopReason, charge, poll};

/// Facts the later region step may consume only as a whole.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proof {
    pub(crate) hash_switch_bci: u32,
    pub(crate) final_switch_bci: u32,
    pub(crate) selector_slot: u16,
    pub(crate) selector_store_bci: u32,
    /// The single-use SSA producer immediately before the selector's local save. The builder must
    /// bind its expression and all transitive producers to the new switch selector before removing
    /// the save; this proof does not erase that expression.
    pub(crate) selector_producer_bci: u32,
    pub(crate) discriminator_slot: u16,
    pub(crate) initial_discriminator: i64,
    /// Literal and final integer key in first-switch traversal order.
    pub(crate) labels: Vec<(String, i64)>,
    /// Explicit final keys with no producer. Only a key sharing the default target can be omitted.
    pub(crate) default_holes: Vec<i64>,
    pub(crate) owned_bcis: BTreeSet<u32>,
}

/// Return a complete certificate or a plain refusal. Budget/cancellation always propagates.
///
/// The accepted grammar is intentionally narrower than all legal String switches: one saved
/// discriminator, a direct String hash invocation, and bucket bodies made solely of ordered
/// `equals` tests and constant writes. This covers javac's ordinary and colliding buckets while
/// leaving unfamiliar lowerings on the already correct integer path.
pub(crate) fn prove(
    ir: &MethodIr,
    hash_switch_bci: u32,
    final_switch_bci: u32,
    budget: &mut Budget,
) -> Result<Option<Proof>, StopReason> {
    let (Some(code), Some(ssa), Some(canonical)) = (ir.code(), ir.ssa(), ir.canonical()) else {
        return Ok(None);
    };
    if code.stopped_at.is_some() || hash_switch_bci >= final_switch_bci {
        return Ok(None);
    }
    charge(
        budget,
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(code.instructions.len()).unwrap_or(u64::MAX),
        Some(hash_switch_bci),
    )?;
    let ops = Operations::of(code, ir.constant_pool());
    let bcis: Vec<u32> = code.instructions.iter().map(|item| item.bci).collect();
    let positions: BTreeMap<u32, usize> =
        bcis.iter().enumerate().map(|(i, bci)| (*bci, i)).collect();
    let Some(&hash_at) = positions.get(&hash_switch_bci) else {
        return Ok(None);
    };
    let Some(&final_at) = positions.get(&final_switch_bci) else {
        return Ok(None);
    };
    let Some((hash_cases, hash_default)) = ops.get(hash_switch_bci).and_then(Operation::switch)
    else {
        return Ok(None);
    };
    let Some((final_cases, final_default)) = ops.get(final_switch_bci).and_then(Operation::switch)
    else {
        return Ok(None);
    };
    if hash_cases.is_empty() || hash_cases.len() > 256 || final_cases.len() > 256 {
        return Ok(None);
    }
    if final_cases
        .iter()
        .map(|(key, _)| *key)
        .collect::<BTreeSet<_>>()
        .len()
        != final_cases.len()
    {
        return Ok(None);
    }
    let Some(Operation::Load {
        slot: discriminator_slot,
    }) = final_at.checked_sub(1).and_then(|i| ops.get(bcis[i]))
    else {
        return Ok(None);
    };
    let join_at = final_at - 1;
    let join_bci = bcis[join_at];
    if hash_default != join_bci {
        return Ok(None);
    }
    // A carried operand below the hash/equals calls could be consumed after the second switch.
    // The javac lowering enters each bucket and the discriminator read with an empty stack.
    if hash_cases
        .iter()
        .map(|(_, target)| *target)
        .chain([join_bci])
        .any(|entry| {
            ssa.blocks()
                .iter()
                .find(|block| block.block().bci() == entry)
                .is_none_or(|block| {
                    block
                        .entry()
                        .iter()
                        .any(|(slot, _)| matches!(slot, Slot::Stack(_)))
                })
        })
    {
        return Ok(None);
    }

    // The hash value is either still on the stack or is saved and immediately reloaded. No
    // arithmetic, other call, or second consumer can fit between its definition and dispatch.
    let (hash_call_at, hash_slot) = match hash_at.checked_sub(1).and_then(|i| ops.get(bcis[i])) {
        Some(Operation::Invoke(call)) if is_string_call(call, "hashCode", "()I") => {
            (hash_at - 1, None)
        }
        Some(Operation::Load { slot }) if hash_at >= 3 => {
            if !matches!(ops.get(bcis[hash_at - 2]), Some(Operation::Store { slot: stored }) if stored == slot)
                || !matches!(ops.get(bcis[hash_at - 3]), Some(Operation::Invoke(call)) if is_string_call(call, "hashCode", "()I"))
            {
                return Ok(None);
            }
            (hash_at - 3, Some(*slot))
        }
        _ => return Ok(None),
    };
    let Some(Operation::Load {
        slot: selector_slot,
    }) = hash_call_at.checked_sub(1).and_then(|i| ops.get(bcis[i]))
    else {
        return Ok(None);
    };
    if selector_slot == discriminator_slot
        || hash_slot.is_some_and(|slot| slot == *selector_slot || slot == *discriminator_slot)
    {
        return Ok(None);
    }
    let Some(init_store_at) = hash_call_at.checked_sub(2) else {
        return Ok(None);
    };
    let Some(init_push_at) = hash_call_at.checked_sub(3) else {
        return Ok(None);
    };
    let Some(selector_store_at) = init_push_at.checked_sub(1) else {
        return Ok(None);
    };
    if !matches!(ops.get(bcis[selector_store_at]), Some(Operation::Store { slot }) if slot == selector_slot)
    {
        return Ok(None);
    }
    let Some(Operation::Store { slot }) = ops.get(bcis[init_store_at]) else {
        return Ok(None);
    };
    if slot != discriminator_slot {
        return Ok(None);
    }
    let Some(Operation::Push(ConstantValue::Int(initial_discriminator))) =
        ops.get(bcis[init_push_at])
    else {
        return Ok(None);
    };
    if final_cases
        .iter()
        .any(|(key, _)| key == initial_discriminator)
    {
        return Ok(None);
    }

    let mut owned_bcis: BTreeSet<u32> = bcis[init_push_at..=hash_at].iter().copied().collect();
    owned_bcis.insert(bcis[selector_store_at]);
    owned_bcis.insert(join_bci);
    owned_bcis.insert(final_switch_bci);
    let mut labels = Vec::new();
    let mut seen_hashes = BTreeSet::new();
    let mut seen_literals = BTreeSet::new();
    let mut seen_discriminators = BTreeSet::new();
    let mut ordered = hash_cases.to_vec();
    ordered.sort_by_key(|(_, target)| *target);
    if ordered.iter().any(|(key, target)| {
        !seen_hashes.insert(*key) || *target <= hash_switch_bci || *target >= join_bci
    }) || ordered.windows(2).any(|pair| pair[0].1 == pair[1].1)
    {
        return Ok(None);
    }
    for (bucket_index, (hash, start)) in ordered.iter().enumerate() {
        poll(budget, Some(*start))?;
        let Some(&mut_start) = positions.get(start) else {
            return Ok(None);
        };
        let end_bci = ordered
            .get(bucket_index + 1)
            .map_or(join_bci, |(_, next)| *next);
        let Some(&end) = positions.get(&end_bci) else {
            return Ok(None);
        };
        let mut cursor = mut_start;
        if cursor >= end {
            return Ok(None);
        }
        loop {
            poll(budget, Some(bcis[cursor]))?;
            if cursor + 5 >= end {
                return Ok(None);
            }
            if !matches!(ops.get(bcis[cursor]), Some(Operation::Load { slot }) if slot == selector_slot)
            {
                return Ok(None);
            }
            let Some(Operation::Push(ConstantValue::String(literal))) = ops.get(bcis[cursor + 1])
            else {
                return Ok(None);
            };
            if !matches!(ops.get(bcis[cursor + 2]), Some(Operation::Invoke(call)) if is_string_call(call, "equals", "(Ljava/lang/Object;)Z"))
            {
                return Ok(None);
            }
            let Some(Operation::Comparison {
                op: CompareOp::JumpIfZero,
                target: false_target,
            }) = ops.get(bcis[cursor + 3])
            else {
                return Ok(None);
            };
            let Some(Operation::Push(ConstantValue::Int(key))) = ops.get(bcis[cursor + 4]) else {
                return Ok(None);
            };
            // `decode::constant` currently uses a lossy Modified UTF-8 conversion. U+FFFD
            // therefore cannot distinguish an actual replacement character from a malformed
            // or isolated UTF-16 surrogate; neither hash nor Java literal text is safe to claim.
            if !matches!(ops.get(bcis[cursor + 5]), Some(Operation::Store { slot }) if slot == discriminator_slot)
                || literal.contains('\u{fffd}')
                || java_hash(literal) != *hash as i32
                || !seen_literals.insert(literal.clone())
                || !seen_discriminators.insert(*key)
                || !final_cases.iter().any(|(final_key, _)| final_key == key)
            {
                return Ok(None);
            }
            labels.push((literal.clone(), *key));
            for bci in &bcis[cursor..=cursor + 5] {
                owned_bcis.insert(*bci);
            }
            cursor += 6;
            if cursor < end && matches!(ops.get(bcis[cursor]), Some(Operation::Transfer)) {
                if branch_target(code, cursor) != Some(join_bci) {
                    return Ok(None);
                }
                owned_bcis.insert(bcis[cursor]);
                cursor += 1;
            }
            if *false_target == join_bci {
                if cursor != end {
                    return Ok(None);
                }
                break;
            }
            if Some(*false_target) != bcis.get(cursor).copied() || cursor >= end {
                return Ok(None);
            }
        }
    }
    if labels.is_empty()
        || owned_bcis.iter().any(|bci| {
            *bci > hash_switch_bci && *bci < final_switch_bci && !positions.contains_key(bci)
        })
    {
        return Ok(None);
    }
    // Every instruction in the removed interval has been accounted for. In particular, an extra
    // hash consumer or an effect in a bucket prevents publication even if the labels look right.
    if bcis[hash_at..=final_at]
        .iter()
        .any(|bci| !owned_bcis.contains(bci))
    {
        return Ok(None);
    }
    if let Some(hash_slot) = hash_slot
        && bcis[hash_call_at + 1..final_at].iter().any(|bci| matches!(ops.get(*bci), Some(Operation::Load { slot }) if *slot == hash_slot && *bci != bcis[hash_at - 1])) {
        return Ok(None);
    }
    if bcis[hash_call_at..final_at].iter().any(
        |bci| matches!(ops.get(*bci), Some(Operation::Store { slot }) if slot == selector_slot),
    ) {
        return Ok(None);
    }
    if bcis[init_store_at + 1..].iter().any(|bci| matches!(ops.get(*bci), Some(Operation::Load { slot }) if slot == discriminator_slot && *bci != join_bci)) {
        return Ok(None);
    }
    // The hash result's SSA value must have exactly the consumer represented above. This catches
    // a duplicated stack value as well as a saved local value with another user.
    let expected_hash_consumer = hash_slot.map_or(hash_switch_bci, |_| bcis[hash_call_at + 1]);
    let hash_values: Vec<_> = ssa.values_with_ids().filter(|(_, value)| matches!(value.def(), Definition::Instruction { bci, .. } if *bci == bcis[hash_call_at])).collect();
    if hash_values.len() != 1
        || hash_values[0].1.uses().len() != 1
        || hash_values[0].1.uses()[0].bci() != Some(expected_hash_consumer)
    {
        return Ok(None);
    }
    if let Some(_hash_slot) = hash_slot {
        let saved_hash: Vec<_> = ssa.values_with_ids().filter(|(_, value)| matches!(value.def(), Definition::Instruction { bci, .. } if *bci == bcis[hash_call_at + 1])).collect();
        if saved_hash.len() != 1
            || saved_hash[0].1.uses().len() != 1
            || saved_hash[0].1.uses()[0].bci() != Some(bcis[hash_at - 1])
        {
            return Ok(None);
        }
    }
    let selector_values: Vec<_> = ssa.values_with_ids().filter(|(_, value)| matches!(value.def(), Definition::Instruction { bci, .. } if *bci == bcis[selector_store_at])).collect();
    let expected_selector_uses: BTreeSet<_> = owned_bcis
        .iter()
        .copied()
        .filter(
            |bci| matches!(ops.get(*bci), Some(Operation::Load { slot }) if slot == selector_slot),
        )
        .collect();
    let actual_selector_uses: Vec<Option<u32>> = selector_values
        .first()
        .map(|(_, value)| value.uses().iter().map(|site| site.bci()).collect())
        .unwrap_or_default();
    let actual_instruction_uses: Vec<u32> =
        actual_selector_uses.iter().flatten().copied().collect();
    let actual_instruction_set: BTreeSet<u32> = actual_instruction_uses.iter().copied().collect();
    if selector_values.len() != 1
        || actual_instruction_uses.len() != actual_instruction_set.len()
        || actual_instruction_set != expected_selector_uses
    {
        return Ok(None);
    }
    let selector_value = selector_values[0].0;
    if ssa.phis().iter().any(|phi| {
        phi.inputs().contains(&PhiInput::Value(selector_value))
            && (phi
                .inputs()
                .iter()
                .any(|input| *input != PhiInput::Value(selector_value))
                || ssa.value(phi.value()).replaced_by() != Some(selector_value))
    }) {
        return Ok(None);
    }
    let expected_phi_uses: usize = ssa
        .phis()
        .iter()
        .map(|phi| {
            phi.inputs()
                .iter()
                .filter(|input| **input == PhiInput::Value(selector_value))
                .count()
        })
        .sum();
    if actual_selector_uses
        .iter()
        .filter(|site| site.is_none())
        .count()
        != expected_phi_uses
    {
        return Ok(None);
    }
    let selector_store_read = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| instruction.bci() == bcis[selector_store_at])
        .and_then(|instruction| instruction.reads().last().map(|(_, value)| *value));
    if selector_store_read.is_none_or(|value| {
        let uses = ssa.value(value).uses();
        uses.len() != 1 || uses[0].bci() != Some(bcis[selector_store_at])
            || !matches!(ssa.value(value).def(), Definition::Instruction { bci, .. } if selector_store_at > 0 && *bci == bcis[selector_store_at - 1])
    }) {
        return Ok(None);
    }
    // A branch elsewhere may not enter the comparison subgraph or discriminator read. A region
    // may only claim the first hash switch as an entry; no hidden predecessor is discarded.
    for (index, bci) in bcis.iter().enumerate() {
        poll(budget, Some(*bci))?;
        let target = match ops.get(*bci) {
            Some(Operation::Comparison { target, .. }) => Some(*target),
            Some(Operation::Transfer) => branch_target(code, index),
            _ => None,
        };
        if let Some(target) = target
            && target >= bcis[selector_store_at]
            && target <= final_switch_bci
            && (target <= hash_switch_bci || !owned_bcis.contains(bci))
        {
            return Ok(None);
        }
        if let Some((cases, default)) = ops.get(*bci).and_then(Operation::switch)
            && *bci != hash_switch_bci
            && cases
                .iter()
                .map(|(_, target)| *target)
                .chain([default])
                .any(|target| target >= bcis[selector_store_at] && target <= final_switch_bci)
        {
            return Ok(None);
        }
    }
    if canonical.blocks().iter().any(|block| {
        block.id().is_clone()
            && block.id().bci() >= bcis[selector_store_at]
            && block.id().bci() <= final_switch_bci
    }) {
        return Ok(None);
    }
    let owned_blocks: BTreeSet<_> = ssa
        .effects()
        .instructions()
        .iter()
        .filter(|effect| {
            effect.bci() >= bcis[selector_store_at] && effect.bci() <= final_switch_bci
        })
        .map(|effect| effect.block().clone())
        .collect();
    let Some(entry_block) = ssa
        .effects()
        .instructions()
        .iter()
        .find(|effect| effect.bci() == bcis[selector_store_at])
        .map(|effect| effect.block())
    else {
        return Ok(None);
    };
    if canonical.edges().iter().any(|edge| {
        owned_blocks.contains(edge.to())
            && !owned_blocks.contains(edge.from())
            && edge.to() != entry_block
    }) {
        return Ok(None);
    }
    let hash_handlers = ssa
        .effects()
        .instructions()
        .iter()
        .find(|effect| effect.bci() == bcis[hash_call_at])
        .map(|effect| effect.handlers());
    if hash_handlers.is_none()
        || ssa.effects().instructions().iter().any(|effect| {
            owned_bcis.contains(&effect.bci())
                && effect.may_throw()
                && Some(effect.handlers()) != hash_handlers
        })
    {
        return Ok(None);
    }
    let default_holes: Vec<i64> = final_cases
        .iter()
        .filter(|(key, _)| !seen_discriminators.contains(key))
        .map(|(key, _)| *key)
        .collect();
    if default_holes.iter().any(|hole| {
        final_cases
            .iter()
            .find(|(key, _)| key == hole)
            .is_none_or(|(_, target)| *target != final_default)
    }) {
        return Ok(None);
    }
    Ok(Some(Proof {
        hash_switch_bci,
        final_switch_bci,
        selector_slot: *selector_slot,
        selector_store_bci: bcis[selector_store_at],
        selector_producer_bci: bcis[selector_store_at - 1],
        discriminator_slot: *discriminator_slot,
        initial_discriminator: *initial_discriminator,
        labels,
        default_holes,
        owned_bcis,
    }))
}

fn is_string_call(call: &crate::facts::CallTarget, name: &str, descriptor: &str) -> bool {
    call.kind() == InvokeKind::Virtual
        && !call.is_interface_reference()
        && call.owner() == "java/lang/String"
        && call.name() == name
        && call.descriptor() == descriptor
}

fn branch_target(code: &jarde_reader::classfile::MethodCodeFacts, index: usize) -> Option<u32> {
    let instruction = code.instructions.get(index)?;
    let offset = code.operands().get(index)?.branch_offset?;
    instruction.bci.checked_add_signed(offset)
}

/// Java 8 `String.hashCode` is defined over UTF-16 code units, not Unicode scalar values.
fn java_hash(value: &str) -> i32 {
    value.encode_utf16().fold(0i32, |hash, unit| {
        hash.wrapping_mul(31).wrapping_add(i32::from(unit))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jarde_jvm::engine::analyze_method_ir;
    use jarde_jvm::environment::ResolutionEnvironment;
    use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
    use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
    use jarde_reader::budget::Limits;
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant,
    };
    use jarde_reader::view::{
        DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, ModuleMode, MultiReleasePolicy,
        PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
    };

    const UNICODE: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-24/string-switch-unicode/StringSwitchUnicode.class"
    );
    const BASIC: &[u8] =
        include_bytes!("../../../tests/fixtures/p3-string-switch/v8/StringSwitchProbe.class");
    const MIDDLE_DEFAULT: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-24/string-switch-variants/StringSwitchMiddleDefault.class"
    );
    const EXTRA_HASH: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-24/string-switch-negative-fixtures/classes/ExtraHashUse.class"
    );
    const WRONG_BUCKET: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-24/string-switch-negative-fixtures/classes/WrongHashBucket.class"
    );
    const BUCKET_EFFECT: &[u8] =
        include_bytes!("../../../tests/fixtures/p3-string-switch/v8/ExtraBucketEffect.class");

    fn certificate(class: &[u8], owner: &str, hash: u32, final_switch: u32) -> Option<Proof> {
        let mut budget = Budget::new(Limits {
            input_bytes: 1 << 20,
            archive_entries: 1_000,
            entry_bytes: 1 << 20,
            read_bytes: 1 << 20,
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            output_bytes: 1 << 20,
            class_headers: 10,
            method_bodies: 10,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
            normalization_clones: 1 << 20,
            nested_depth: 8,
            dependency_depth: 4,
            elapsed_millis: u64::MAX,
        });
        let snapshot =
            ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget).unwrap();
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class).to_hex().to_string()),
                length: class.len() as u64,
            },
            variant: PhysicalVariant::Base,
        };
        let descriptor = if owner == "StringSwitchUnicode" {
            b"(Ljava/lang/String;)Ljava/lang/String;".as_slice()
        } else {
            b"(Ljava/lang/String;)I".as_slice()
        };
        let method = PhysicalMethodId {
            owner: definition,
            name: JvmBytes(b"choose".to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        };
        let domain = LoadDomain {
            loader: jarde_reader::view::LoaderId("app".to_owned()),
            parent_loader: None,
            delegation: DelegationPolicy::ParentFirst,
            roots: vec![LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            }],
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        };
        let environment = ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        };
        let request = MethodAnalysisRequest {
            environment,
            method,
            stages: AnalysisStage::ALL.to_vec(),
        };
        let analysis = analyze_method_ir(&[snapshot], &request, &mut budget).unwrap();
        assert!(analysis.ir().code().is_some(), "{owner} analysis has code");
        assert!(analysis.ir().canonical().is_some(), "{owner} normalizes");
        assert!(analysis.ir().ssa().is_some(), "{owner} has consistent SSA");
        prove(analysis.ir(), hash, final_switch, &mut budget).unwrap()
    }

    #[test]
    fn utf16_hash_matches_java_for_bmp_supplementary_and_collision() {
        assert_eq!(java_hash(""), 0);
        assert_eq!(java_hash("雪"), 38634);
        assert_eq!(java_hash("𐐷"), 1770582);
        assert_eq!(java_hash("Aa"), java_hash("BB"));
    }

    #[test]
    fn unicode_and_collision_buckets_have_complete_final_mapping() {
        let basic = certificate(BASIC, "StringSwitchProbe", 8, 76)
            .expect("frozen 635-byte lowering is proved");
        assert_eq!(basic.selector_producer_bci, 0);
        assert_eq!(basic.labels.len(), 3);
        assert!(basic.labels.iter().any(|label| label.0 == "Aa"));
        assert!(basic.labels.iter().any(|label| label.0 == "BB"));
        let proof = certificate(UNICODE, "StringSwitchUnicode", 11, 84)
            .expect("javac Unicode lowering is proved");
        assert_eq!(proof.selector_producer_bci, 1);
        assert_eq!(
            proof.labels,
            [
                ("".to_owned(), 0),
                ("雪".to_owned(), 1),
                ("𐐷".to_owned(), 2)
            ]
        );
        assert!(proof.default_holes.is_empty());
        let proof = certificate(MIDDLE_DEFAULT, "StringSwitchMiddleDefault", 15, 147)
            .expect("collision and shared default are proved");
        assert!(proof.labels.iter().any(|label| label.0 == "Aa"));
        assert!(proof.labels.iter().any(|label| label.0 == "BB"));
        assert_eq!(proof.default_holes, [2]);
    }

    #[test]
    fn independent_hash_use_and_wrong_bucket_refuse() {
        assert!(certificate(EXTRA_HASH, "ExtraHashUse", 18, 88).is_none());
        assert!(certificate(WRONG_BUCKET, "WrongHashBucket", 13, 66).is_none());
        assert!(certificate(BUCKET_EFFECT, "ExtraBucketEffect", 8, 51).is_none());
    }

    #[test]
    fn duplicate_literal_and_cross_bucket_predecessor_refuse() {
        let mut duplicate = BASIC.to_vec();
        let matches: Vec<_> = duplicate
            .windows(2)
            .enumerate()
            .filter(|(_, bytes)| *bytes == b"BB")
            .map(|(at, _)| at)
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "one UTF-8 constant-pool BB literal in the frozen class"
        );
        duplicate[matches[0]..matches[0] + 2].copy_from_slice(b"Aa");
        assert!(certificate(&duplicate, "StringSwitchProbe", 8, 76).is_none());

        let mut predecessor = BASIC.to_vec();
        // BCI 42's `ifeq 50` is encoded as 99 00 08. Redirect its false edge to BCI 64,
        // entering the other hash bucket without selecting that bucket's hash key.
        let matches: Vec<_> = predecessor
            .windows(3)
            .enumerate()
            .filter(|(_, bytes)| *bytes == [0x99, 0, 8])
            .map(|(at, _)| at)
            .collect();
        assert_eq!(matches.len(), 1);
        predecessor[matches[0] + 2] = 22;
        assert!(certificate(&predecessor, "StringSwitchProbe", 8, 76).is_none());

        let mut wrong_hash = BASIC.to_vec();
        // The unique lookup pair 122 -> BCI 64 is encoded as key 122 and offset 56.
        let pair = [0, 0, 0, 122, 0, 0, 0, 56];
        let matches: Vec<_> = wrong_hash
            .windows(pair.len())
            .enumerate()
            .filter(|(_, bytes)| *bytes == pair)
            .map(|(at, _)| at)
            .collect();
        assert_eq!(matches.len(), 1);
        wrong_hash[matches[0] + 3] = 123;
        assert!(certificate(&wrong_hash, "StringSwitchProbe", 8, 76).is_none());

        let mut replacement_literal = UNICODE.to_vec();
        let snow_utf8 = [0xe9, 0x9b, 0xaa];
        let matches: Vec<_> = replacement_literal
            .windows(snow_utf8.len())
            .enumerate()
            .filter(|(_, bytes)| *bytes == snow_utf8)
            .map(|(at, _)| at)
            .collect();
        assert_eq!(matches.len(), 1);
        replacement_literal[matches[0]..matches[0] + 3].copy_from_slice(&[0xef, 0xbf, 0xbd]);
        // Keep the bucket correct for U+FFFD so the lossy-spelling gate is the refusal reason.
        let pair = [0, 0, 0x96, 0xea, 0, 0, 0, 47];
        let matches: Vec<_> = replacement_literal
            .windows(pair.len())
            .enumerate()
            .filter(|(_, bytes)| *bytes == pair)
            .map(|(at, _)| at)
            .collect();
        assert_eq!(matches.len(), 1);
        replacement_literal[matches[0] + 2..matches[0] + 4].copy_from_slice(&[0xff, 0xfd]);
        assert!(certificate(&replacement_literal, "StringSwitchUnicode", 11, 84).is_none());
    }

    #[test]
    fn final_switch_external_predecessor_refuses() {
        let mut class = MIDDLE_DEFAULT.to_vec();
        // Final arm BCI 192 normally jumps to the method tail at 222. Redirect it to the
        // discriminator load at 146. This is a valid same-stack-shape back edge from outside
        // the owned comparison subgraph into the final switch.
        let matches: Vec<_> = class
            .windows(3)
            .enumerate()
            .filter(|(_, bytes)| *bytes == [0xa7, 0, 30])
            .map(|(at, _)| at)
            .collect();
        assert_eq!(matches.len(), 1);
        class[matches[0] + 1..matches[0] + 3].copy_from_slice(&(-46i16).to_be_bytes());
        assert!(certificate(&class, "StringSwitchMiddleDefault", 15, 147).is_none());
    }
}
