//! The field one body reads or writes (P3 2.3, rule `field@1`).
//!
//! # What this rule presents, and what proves it
//!
//! A `getfield`/`getstatic`/`putfield`/`putstatic` of the presented body becomes a field access —
//! `receiver.f`, `Type.f`, `receiver.f = value`, `Type.f = value` — only when the run proves **which
//! member** the instruction names:
//!
//! * the instruction itself states the member: the class's own pool names the owner, the field and
//!   its descriptor ([`crate::decode`], rule `field@1`'s IR precondition), and nothing here invents a
//!   spelling from anything else;
//! * for an **instance** access the receiver's static type has to be exactly the member's owner. That
//!   is the one proof that keeps the written text meaning the same field: `a.f` and `b.f` are
//!   different fields when `B extends A` declares its own `f`, so a receiver whose type is merely a
//!   *subtype* would make `receiver.f` name a field this instruction did not read. The frames state
//!   the receiver's type, and the rule compares it with the pool's owner in the class file's own
//!   internal form;
//! * for a **static** access there is no receiver and no shadowing question: the owner type and the
//!   field name are the member, and the text spells them.
//!
//! # The one write that happens *before* the constructor's own call
//!
//! JVMS 4.10.1.9 lets an instance initializer store into a field of the class being constructed
//! **before** the constructor call runs (`aload_0; aload_1; putfield C.f`), which is exactly how
//! javac writes the synthetic reference an inner class keeps to its enclosing instance. The receiver
//! of such a write is the frames' `UninitializedThis`, whose type is not a class name — so the
//! comparison above cannot apply, and this rule takes the frame pass's own rule instead: an
//! `UninitializedThis` may be stored through only a `Fieldref` that **names the class being
//! constructed**, which the caller states as the member's declaring class
//! ([`crate::facts::DeclaringClass`]). A run that does not state it refuses those writes with that
//! requirement named, rather than dropping the assignment or moving it after the constructor call.
//!
//! # Where the text goes, and why nothing moves
//!
//! A read is a value: its text lands where the value is consumed (the store, the call or the
//! `return` that renders it), which is why a read writes no statement of its own — and why
//! [`crate::build`] asks this plan whether a read is claimed before treating it as a reader of
//! another instruction's value. A write is a statement of its own, written where the instruction
//! runs. Neither is ever moved: the order the statements come out in is the bytecode's order, which
//! is the invariant a constructor's initializer sequence depends on (`super(…)` first, then the
//! instance initializers, then the rest of the constructor — JLS 12.5).

use std::collections::BTreeMap;

use jarde_jvm::method_ir::{RefType, SsaInstruction, SsaTable, Value, ValueId};
use serde::Serialize;

use crate::build::stack_operands;
use crate::decode::Operations;
use crate::evidence::Publication;
use crate::facts::{DeclaringClass, FieldAccess, Operation, internal_form};
use crate::pass::{FIELD, Precondition, RuleVersion};
use crate::refusal::{Gap, Refusal};

/// The pass answerable for every verdict of this module.
pub(crate) const RULE: RuleVersion = FIELD.rule();

/// The requirement a write on an uninitialized `this` states.
const DECLARING_CLASS: Precondition = Precondition::Metadata {
    attribute: "declaring_class",
};

/// Every field instruction of one body this rule read, the ones it claimed, and the gaps it states
/// in every selection.
pub(crate) struct Plan {
    claimed: BTreeMap<u32, (Evidence, Shape)>,
    /// Why a field instruction was not presented, in BCI order. A gap is not the optional evidence:
    /// it is what every selection reports about an instruction this rule refused.
    refusals: Vec<Gap>,
    /// The records, when this run published rule records: the presented access of every claimed
    /// instruction and the refusal of every other one, in BCI order.
    records: Vec<FieldRecord>,
}

impl Plan {
    /// An empty plan: a body that reads and writes no field at all.
    pub(crate) fn empty() -> Self {
        Self {
            claimed: BTreeMap::new(),
            refusals: Vec::new(),
            records: Vec::new(),
        }
    }

    /// Whether the instruction at one BCI is a field access this rule claimed.
    ///
    /// This is the question [`crate::build`] asks before it treats a field instruction as a reader
    /// of another instruction's value: a claimed access writes the value it reads into its own
    /// text, and one this rule did not claim is quoted as bytecode and writes nothing at all.
    pub(crate) fn owns(&self, bci: u32) -> bool {
        self.claimed.contains_key(&bci)
    }

    /// The member and the values one claimed field access names, or `None` when this rule refused
    /// the instruction.
    pub(crate) fn claim(&self, bci: u32) -> Option<(&Evidence, &Shape)> {
        self.claimed
            .get(&bci)
            .map(|(evidence, shape)| (evidence, shape))
    }

    /// Every field instruction the rule read and did not present, in BCI order.
    pub(crate) fn refusals(&self) -> &[Gap] {
        &self.refusals
    }

    /// Every field instruction read, presented or refused, in BCI order — the evidence a report
    /// reads back when this run selected rule records.
    pub(crate) fn records(&self) -> &[FieldRecord] {
        &self.records
    }

    /// How many field instructions the rule read, and how many of them it presented. The two counts
    /// are the rule's own work over the whole body, so the report states them whatever the caller
    /// selected and whatever range the selection carried.
    pub(crate) fn counts(&self) -> (u64, u64) {
        let presented = u64::try_from(self.claimed.len()).unwrap_or(u64::MAX);
        let refused = u64::try_from(self.refusals.len()).unwrap_or(u64::MAX);
        (presented + refused, presented)
    }
}

/// The member one field instruction names, as the pool and the instruction itself state it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Evidence {
    pub(crate) bci: u32,
    pub(crate) access: FieldAccess,
    pub(crate) is_static: bool,
    pub(crate) owner: String,
    pub(crate) name: String,
    pub(crate) descriptor: String,
}

/// What one claimed field instruction reads.
pub(crate) struct Shape {
    /// The receiver, for an instance access; `None` for a static one.
    pub(crate) receiver: Option<ValueId>,
    /// The value a write stores; `None` for a read.
    pub(crate) value: Option<ValueId>,
}

impl Shape {
    /// Whether this access writes the field rather than reading it.
    pub(crate) fn writes(&self) -> bool {
        self.value.is_some()
    }
}

/// Reads every field instruction of one body.
///
/// `declaring` is what the caller stated about the class that declares this body, and it is read for
/// exactly one thing: the writes an instance initializer makes on its own `UninitializedThis` before
/// its constructor call, which JVMS 4.10.1.9 allows only through a `Fieldref` that names that class.
pub(crate) fn plan(
    ssa: &SsaTable,
    operations: &Operations,
    declaring: Option<&DeclaringClass>,
    publication: Publication,
) -> Plan {
    let mut plan = Plan::empty();
    for instruction in ssa.blocks().iter().flat_map(|block| block.instructions()) {
        let at = instruction.bci();
        let Some(Operation::Field {
            access,
            is_static,
            owner,
            name,
            descriptor,
        }) = operations.get(at)
        else {
            continue;
        };
        let evidence = Evidence {
            bci: at,
            access: *access,
            is_static: *is_static,
            owner: owner.clone(),
            name: name.clone(),
            descriptor: descriptor.clone(),
        };
        match verify(instruction, &evidence, ssa, declaring) {
            Ok(shape) => {
                // The record is built only when the run publishes rule records *and* the
                // instruction's own BCI is inside the selected driver range.
                if publication.publishes(&[at]) {
                    crate::demand_counts::record_built(
                        crate::evidence::RecoveryEvidenceKind::RuleDetails,
                    );
                    plan.records.push(FieldRecord::of_presented(&evidence));
                }
                plan.claimed.insert(at, (evidence, shape));
            }
            Err(refusal) => {
                // The refusal *shape* is the gap every selection carries; the record that owns it
                // is built only when this run publishes rule records and the instruction's BCI is
                // inside the selected driver range.
                let refusal = FieldRefusal::of(&refusal, evidence.bci);
                plan.refusals
                    .push(Gap::at(refusal.code, evidence.bci, refusal.message.clone()));
                if publication.publishes(&[at]) {
                    crate::demand_counts::record_built(
                        crate::evidence::RecoveryEvidenceKind::RuleDetails,
                    );
                    plan.records
                        .push(FieldRecord::of(&evidence, false, Some(refusal)));
                }
            }
        }
    }
    plan.records.sort_by_key(|record| record.bci);
    plan.refusals.sort_by_key(|gap| gap.position());
    plan
}

/// Verifies one field instruction, or states the link that failed.
fn verify(
    instruction: &SsaInstruction,
    evidence: &Evidence,
    ssa: &SsaTable,
    declaring: Option<&DeclaringClass>,
) -> Result<Shape, Refusal> {
    let at = evidence.bci;
    let operands = stack_operands(instruction);
    let shape = |detail: String| Refusal::shape("jre_field_shape", detail);
    if evidence.is_static {
        // A static field has no receiver: there is no second member of the same name to confuse it
        // with, so the owner and the name the pool states are the whole of what has to be proven.
        let value = match evidence.access {
            FieldAccess::Read => None,
            FieldAccess::Write => match operands.last().copied() {
                Some((_, value)) => Some(value),
                None => {
                    return Err(shape(format!(
                        "the static write at BCI {at} reads no value to store"
                    )));
                }
            },
        };
        return Ok(Shape {
            receiver: None,
            value,
        });
    }
    let Some((_, receiver)) = operands.first().copied() else {
        return Err(shape(format!(
            "the field access at BCI {at} reads no receiver this run states"
        )));
    };
    match stated_type(ssa, receiver) {
        // The receiver's own type is the member's owner: `receiver.f` names this field and no other.
        Some(stated) if stated == evidence.owner => {}
        // The uninitialized `this` of an instance initializer, before its constructor call: the one
        // receiver whose type is not a class name, and which JVMS 4.10.1.9 lets through only a
        // `Fieldref` naming the class being constructed.
        None if matches!(ssa.value(receiver).ty(), Value::UninitializedThis) => {
            let named = declaring.is_some_and(|class| class.name() == evidence.owner);
            if !named {
                return Err(Refusal::unmet(
                    &FIELD,
                    DECLARING_CLASS,
                    match declaring {
                        Some(class) => format!(
                            "the write at BCI {at} stores through `{}`, and the class that declares this constructor is `{}`: JVMS 4.10.1.9 lets the uninitialized `this` be stored through a `Fieldref` that names the class being constructed and no other",
                            evidence.owner,
                            class.name()
                        ),
                        None => format!(
                            "the write at BCI {at} stores through `{}` on the uninitialized `this`, and this run was not told which class declares this constructor: JVMS 4.10.1.9 lets that write reach a `Fieldref` that names the class being constructed, and without the class's own name that cannot be checked",
                            evidence.owner
                        ),
                    },
                ));
            }
        }
        other => {
            return Err(shape(format!(
                "the receiver of the field access at BCI {at} has the type {}, and the member the pool states is a field of `{}`: writing `receiver.{}` would name whichever field the receiver's own type declares, which is not the member this instruction reads",
                other.unwrap_or_else(|| "this run does not state".to_string()),
                evidence.owner,
                evidence.name
            )));
        }
    }
    let value = match evidence.access {
        FieldAccess::Read => None,
        FieldAccess::Write => match operands.get(1).copied() {
            Some((_, value)) => Some(value),
            None => {
                return Err(shape(format!(
                    "the write at BCI {at} reads no value to store"
                )));
            }
        },
    };
    Ok(Shape {
        receiver: Some(receiver),
        value,
    })
}

/// The class name one value's frame entry states, in internal form, when it states one.
fn stated_type(ssa: &SsaTable, value: ValueId) -> Option<String> {
    match ssa.value(value).ty() {
        Value::Ref(RefType::Named { name, .. }) => {
            Some(internal_form(&String::from_utf8_lossy(name)).to_string())
        }
        _ => None,
    }
}

/// One field instruction of the body, presented or refused (P3 2.3).
///
/// The record is the evidence the acceptance asks for: which member the instruction named, whether
/// it read or wrote it, and whether the run presented it as a field access. A refusal names the link
/// of the verification that failed — the receiver whose type the run could not compare, the
/// declaring class it was not told, or a write with no value — so "why is this not a field access"
/// is answered by the run's own record rather than by reading the text.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FieldRecord {
    /// The BCI of the field instruction.
    pub bci: u32,
    /// `"read"` or `"write"`, as the instruction itself does it.
    pub access: &'static str,
    /// Whether the instruction names a static field.
    pub is_static: bool,
    /// The member's owner, in the class file's own internal form.
    pub owner: String,
    /// The member's name.
    pub name: String,
    /// The member's descriptor.
    pub descriptor: String,
    /// Whether the run presented the instruction as a field access.
    pub presented: bool,
    /// Why it did not, when it did not.
    pub refusal: Option<FieldRefusal>,
}

impl FieldRecord {
    /// Whether this instruction was presented as a field access.
    pub fn presented(&self) -> bool {
        self.presented
    }

    /// The rule this record is answerable to: the one that presented the access or refused it.
    pub fn rule(&self) -> RuleVersion {
        RULE
    }

    /// One record of a presented access.
    fn of_presented(evidence: &Evidence) -> Self {
        Self::of(evidence, true, None)
    }

    fn of(evidence: &Evidence, presented: bool, refusal: Option<FieldRefusal>) -> Self {
        Self {
            bci: evidence.bci,
            access: match evidence.access {
                FieldAccess::Read => "read",
                FieldAccess::Write => "write",
            },
            is_static: evidence.is_static,
            owner: evidence.owner.clone(),
            name: evidence.name.clone(),
            descriptor: evidence.descriptor.clone(),
            presented,
            refusal,
        }
    }
}

/// Why one field instruction was not presented as a field access.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FieldRefusal {
    /// The diagnostic code, one `jre_field_*` per link of the verification that can fail.
    pub code: &'static str,
    /// The rule that refused the access.
    pub rule: RuleVersion,
    /// The declared requirement that fell short, when the refusal is one of the rule's own
    /// preconditions rather than a shape that simply is not this one.
    pub requirement: Option<String>,
    /// One sentence stating which link failed, with the instruction's BCI in it.
    pub message: String,
}

impl FieldRefusal {
    /// The refusal of one access, as the report records it.
    fn of(refusal: &Refusal, bci: u32) -> Self {
        Self {
            code: refusal.code(),
            rule: RULE,
            requirement: refusal.requirement().map(Precondition::describe),
            message: format!(
                "the field access at BCI {bci} was not presented: {}",
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
    fn the_rule_states_the_ir_it_reads_and_the_one_declaration_fact_it_needs() {
        assert!(FIELD.requires(Precondition::IrTable(IrTable::Ssa)));
        assert!(FIELD.requires(Precondition::IrTable(IrTable::Code)));
        assert!(FIELD.requires(DECLARING_CLASS));
        assert_eq!(
            FIELD.required_release(),
            None,
            "`x.f` and `x.f = v` are Java in every release"
        );
        assert_eq!(RULE.citation(), "field@1");
    }
}
