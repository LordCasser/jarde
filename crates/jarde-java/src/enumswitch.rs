//! The dispatch table a compiler's `switch` over an enum reads (P3 2.3, rule `enumswitch@1`).
//!
//! # The shape, exactly as far as the evidence goes
//!
//! For `switch (e) { case A: … }` a compiler does not branch on the enum constant; it builds a
//! synthetic `int[]` table indexed by the constant's ordinal and branches on the element:
//!
//! ```text
//! getstatic S.$SwitchMap$T [I ─ e.ordinal() ─ iaload ─ tableswitch { … }
//! ```
//!
//! What this rule verifies is the **read**: a `getstatic` of a static field whose descriptor is
//! `[I`, indexed by the result of an instance call whose descriptor is `()I` — both facts the decode
//! and the pool of this run state, and neither a guess about what the call is *called*. Its output
//! is that read, written where the value is consumed, which for this shape is the selector of the
//! `switch`.
//!
//! # What it deliberately does not claim, and why
//!
//! It does not write `switch (e)` with `case T.CONST:` labels, because the two facts that spelling
//! needs are not in this run:
//!
//! * **which enum constants the table's indices stand for** — the enum class's own fields, which are
//!   a class-level fact of *another* class;
//! * **that the entries are the constants' dense index** — the table's contents, written by the
//!   *synthetic* class's own static initializer, again a different class's body.
//!
//! Neither is the field's name: `$SwitchMap$…` is a compiler convention, and a convention is not
//! evidence — the same discipline that keeps `lambda$…` and `access$…` names from deciding their
//! shapes ([`crate::lambda`], [`crate::accessor`]). So what is presented is the table read the
//! bytecode really performs, which selects exactly what the source's `switch` selected, and the run
//! states in its record and its diagnostics that the constant mapping is not claimed rather than
//! inventing labels for it. A read that is *not* this shape is refused and quoted.

use std::collections::BTreeMap;

use jarde_jvm::method_ir::{Definition, SsaTable, ValueId};
use serde::Serialize;

use crate::build::stack_operands;
use crate::decode::Operations;
use crate::evidence::Publication;
use crate::facts::{FieldAccess, InvokeKind, Operation};
use crate::pass::{ENUMSWITCH, Precondition, RuleVersion};
use crate::refusal::{Gap, Refusal};

/// The pass answerable for every verdict of this module.
pub(crate) const RULE: RuleVersion = ENUMSWITCH.rule();

/// The table read one body performs, the ones it does not, and the gaps it states in every
/// selection.
pub(crate) struct Plan {
    claimed: BTreeMap<u32, (TableRead, IndexCall)>,
    /// Why a candidate read was not presented, in BCI order: what every selection reports about the
    /// rule's refusals, beside the records only a selected run materializes.
    refusals: Vec<Gap>,
    /// The records, when this run published rule records.
    records: Vec<EnumSwitchRecord>,
}

impl Plan {
    /// An empty plan: a body that reads no `int[]` element at all.
    pub(crate) fn empty() -> Self {
        Self {
            claimed: BTreeMap::new(),
            refusals: Vec::new(),
            records: Vec::new(),
        }
    }

    /// Whether the instruction at one BCI is an array read this rule claimed.
    ///
    /// This is the question [`crate::build`] asks before it treats the read as a reader of another
    /// instruction's value: a claimed read writes the values it reads into its own text, and one no
    /// rule claimed is quoted and writes nothing.
    pub(crate) fn owns(&self, bci: u32) -> bool {
        self.claimed.contains_key(&bci)
    }

    /// The table and the call one claimed read names, or `None` when this rule refused it.
    pub(crate) fn claim(&self, bci: u32) -> Option<(&TableRead, &IndexCall)> {
        self.claimed.get(&bci).map(|(table, index)| (table, index))
    }

    /// Every candidate read the rule refused, in BCI order.
    pub(crate) fn refusals(&self) -> &[Gap] {
        &self.refusals
    }

    /// Every candidate read, presented or refused, in BCI order — the evidence a report reads back
    /// when this run selected rule records.
    pub(crate) fn records(&self) -> &[EnumSwitchRecord] {
        &self.records
    }

    /// How many candidate reads the rule read, and how many of them it presented.
    pub(crate) fn counts(&self) -> (u64, u64) {
        let presented = u64::try_from(self.claimed.len()).unwrap_or(u64::MAX);
        let refused = u64::try_from(self.refusals.len()).unwrap_or(u64::MAX);
        (presented + refused, presented)
    }
}

/// Reads every candidate dispatch-table read of one body.
///
/// The decision is taken for every selection; `publication` gates the *records* only — a record the
/// request did not select, or one whose read is outside the selected driver range, is not built —
/// while the refusal of a candidate is stated as a gap in every selection.
pub(crate) fn plan(ssa: &SsaTable, operations: &Operations, publication: Publication) -> Plan {
    let mut plan = Plan::empty();
    for instruction in ssa.blocks().iter().flat_map(|block| block.instructions()) {
        let at = instruction.bci();
        if operations.get(at) != Some(&Operation::ArrayLoad) {
            continue;
        }
        match verify(instruction, ssa, operations) {
            Ok((table, index)) => {
                if publication.publishes(&[at]) {
                    crate::demand_counts::record_built(
                        crate::evidence::RecoveryEvidenceKind::RuleDetails,
                    );
                    plan.records.push(EnumSwitchRecord {
                        read: at,
                        table: Some(table.clone()),
                        index: Some(index.clone()),
                        presented: true,
                        refusal: None,
                    });
                }
                plan.claimed.insert(at, (table, index));
            }
            Err(refusal) => {
                let refusal = EnumSwitchRefusal::of(&refusal, at);
                plan.refusals
                    .push(Gap::at(refusal.code, at, refusal.message.clone()));
                if publication.publishes(&[at]) {
                    crate::demand_counts::record_built(
                        crate::evidence::RecoveryEvidenceKind::RuleDetails,
                    );
                    plan.records.push(EnumSwitchRecord {
                        read: at,
                        table: None,
                        index: None,
                        presented: false,
                        refusal: Some(refusal),
                    });
                }
            }
        }
    }
    plan.records.sort_by_key(|record| record.read);
    plan.refusals.sort_by_key(|gap| gap.position());
    plan
}

/// Verifies one array read as a dispatch-table read, or states the link that failed.
fn verify(
    instruction: &jarde_jvm::method_ir::SsaInstruction,
    ssa: &SsaTable,
    operations: &Operations,
) -> Result<(TableRead, IndexCall), Refusal> {
    let at = instruction.bci();
    let shape = |detail: String| Refusal::shape("jre_enumswitch_shape", detail);
    let operands = stack_operands(instruction);
    if operands.len() != 2 {
        return Err(shape(format!(
            "the array read at BCI {at} reads {} value(s), and a dispatch-table read reads the table and the index",
            operands.len()
        )));
    }
    let (_, array) = operands[0];
    let (_, index) = operands[1];
    // The table: a **static** field of descriptor `[I`, named by the class's own pool.
    let table = match producer(ssa, array, operations) {
        Some((
            bci,
            Operation::Field {
                access: FieldAccess::Read,
                is_static: true,
                owner,
                name,
                descriptor,
            },
        )) => {
            if descriptor != "[I" {
                return Err(shape(format!(
                    "the array read at BCI {at} reads `{owner}.{name}` of descriptor `{descriptor}`, and a dispatch table is an `int[]`"
                )));
            }
            TableRead {
                bci,
                owner: owner.clone(),
                name: name.clone(),
                descriptor: descriptor.clone(),
            }
        }
        Some((bci, _)) => {
            return Err(shape(format!(
                "the array the read at BCI {at} indexes is produced at BCI {bci}, which is not a static field of this run's pool"
            )));
        }
        None => {
            return Err(shape(format!(
                "the array the read at BCI {at} indexes was not produced by an instruction of this body, so which table it reads is not stated"
            )));
        }
    };
    // The index: the result of an instance call that returns an `int` — the ordinal a compiler reads
    // the case index out of. Whether that call is *called* `ordinal` is not part of the shape.
    let index_call = match producer(ssa, index, operations) {
        Some((bci, Operation::Invoke(target))) => {
            if target.descriptor() != "()I"
                || !matches!(target.kind(), InvokeKind::Virtual | InvokeKind::Interface)
            {
                return Err(shape(format!(
                    "the read at BCI {at} is indexed by the call at BCI {bci} (`{}`), and a dispatch table is indexed by the result of an instance call returning an `int`",
                    target.descriptor()
                )));
            }
            IndexCall {
                bci,
                owner: target.owner().to_string(),
                name: target.name().to_string(),
                descriptor: target.descriptor().to_string(),
            }
        }
        Some((bci, _)) => {
            return Err(shape(format!(
                "the read at BCI {at} is indexed by the value produced at BCI {bci}, which is not a call"
            )));
        }
        None => {
            return Err(shape(format!(
                "the read at BCI {at} is indexed by a value no instruction of this body produced"
            )));
        }
    };
    Ok((table, index_call))
}

/// The instruction that produced one value, with its operation.
fn producer<'a>(
    ssa: &SsaTable,
    value: ValueId,
    operations: &'a Operations,
) -> Option<(u32, &'a Operation)> {
    let Definition::Instruction { bci, .. } = ssa.value(value).def() else {
        return None;
    };
    operations.get(*bci).map(|operation| (*bci, operation))
}

/// One static `int[]` field the run read as a dispatch table.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TableRead {
    /// The BCI of the `getstatic`.
    pub bci: u32,
    /// The member's owner, in internal form, as the pool states it.
    pub owner: String,
    /// The member's name — evidence, never the shape: `$SwitchMap$…` is the compiler's convention.
    pub name: String,
    /// The member's descriptor, `[I` for a table this rule presents.
    pub descriptor: String,
}

/// The call one dispatch-table read is indexed by.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IndexCall {
    /// The BCI of the call.
    pub bci: u32,
    /// The call's owner, in internal form.
    pub owner: String,
    /// The call's name — evidence, never the shape.
    pub name: String,
    /// The call's descriptor, `()I` for an index this rule presents.
    pub descriptor: String,
}

/// What one candidate dispatch-table read was presented as, or why it was not (P3 2.3).
///
/// The record states the table and the call with their own BCIs, so "which `int[]` was this, indexed
/// by what" is answered by the run rather than by reading the text. What the record deliberately
/// does **not** state is a mapping to enum constants: that is another class's declaration, and this
/// run never read it (see the module documentation).
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EnumSwitchRecord {
    /// The BCI of the array read.
    pub read: u32,
    /// The static `int[]` field it reads, when the walk reached one.
    pub table: Option<TableRead>,
    /// The call it is indexed by, when the walk reached one.
    pub index: Option<IndexCall>,
    /// Whether the read was presented as the table read it is.
    pub presented: bool,
    /// Why it was not, when it was not.
    pub refusal: Option<EnumSwitchRefusal>,
}

impl EnumSwitchRecord {
    /// Whether this read was presented.
    pub fn presented(&self) -> bool {
        self.presented
    }

    /// The rule this record is answerable to.
    pub fn rule(&self) -> RuleVersion {
        RULE
    }
}

/// Why one array read was not presented as a dispatch-table read.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EnumSwitchRefusal {
    /// The diagnostic code, one `jre_enumswitch_*` per link of the verification that can fail.
    pub code: &'static str,
    /// The rule that refused the read.
    pub rule: RuleVersion,
    /// The declared requirement that fell short, when the refusal is one of the rule's own
    /// preconditions.
    pub requirement: Option<String>,
    /// One sentence stating which link failed, with the read's BCI in it.
    pub message: String,
}

impl EnumSwitchRefusal {
    fn of(refusal: &Refusal, read: u32) -> Self {
        Self {
            code: refusal.code(),
            rule: RULE,
            requirement: refusal.requirement().map(Precondition::describe),
            message: format!(
                "the array read at BCI {read} was not presented: {}",
                refusal.message()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pass::IrTable;

    #[test]
    fn the_rule_states_the_tables_it_reads_and_claims_no_mapping() {
        assert!(ENUMSWITCH.requires(Precondition::IrTable(IrTable::Ssa)));
        assert!(ENUMSWITCH.requires(Precondition::IrTable(IrTable::Code)));
        assert_eq!(
            ENUMSWITCH.required_release(),
            None,
            "a `switch` and an array read are Java in every release; the shape is the input's"
        );
        assert_eq!(RULE.citation(), "enumswitch@1");
    }
}
