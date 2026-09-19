//! The synthetic accessor a compiler generated: which call site this layer may present as a direct
//! field access, and which it must leave as a call (P3 2.2, rule `accessor@1`, acceptance A12).
//!
//! # Why the callee's own body is the only evidence
//!
//! `access$100(x)` says nothing about itself. The name is a convention, and a class that happens to
//! declare a member called `access$100` is not evidence of anything — so this rule reads the
//! **member's own declaration and body**: it must be `static` and `synthetic` (that is what an
//! accessor is, and what a source-level member is not), and its body must be *exactly* one field
//! access plus the return of what it read or the end of what it wrote:
//!
//! ```text
//! read:   load 0 ─ getfield C.f:D ─ return          descriptor (LC;)D
//! write:  load 0 ─ load 1 ─ putfield C.f:D ─ return descriptor (LC;D)V
//! ```
//!
//! Nothing else may be in it: an accessor that computes, casts, calls or writes twice is refused,
//! and the call site keeps the call it had. The marker `access$` is read for **one** purpose and
//! never for this one: it decides whether a call site that this rule could *not* decide is worth a
//! refusal record (a call to something that looks like an accessor but whose member the run does
//! not hold). Whether a call is presented as a field access is decided by the member's body alone —
//! the same discipline as `lambda@1`, where the `lambda$` marker decides between two equivalent
//! writings and never whether a site is a lambda.
//!
//! # Where the member comes from, and why that is not a second opinion
//!
//! The member is a different method of the same class, so one method's payload cannot hold it: the
//! caller hands over the class's other members as *the same two things the payload holds for the
//! presented body* — the declaration facts and the decoded body — and this module is the only place
//! that reads a meaning into them ([`crate::facts::MemberBody`]). The member's body is decoded
//! against the same class's pool, which is the pool of the presented run, because the member is
//! declared in the same class file. A caller therefore cannot state "this is a getter"; it can only
//! hand over bytes and let the rule decide.
//!
//! # What A12 asks for, and what this rule does not do
//!
//! X1's facts — the caller's call to the accessor and the accessor's read of the field — are the
//! artifact's own and are never written back: presenting the call as `x.f` is a *derived* reading,
//! recorded in the segment table with both original BCIs (the call site's, and the field access in
//! the accessor's body). Both original edges survive exactly as they were; nothing here rewrites,
//! merges or drops either of them.

use jarde_reader::classfile::CpEntryFacts;
use serde::Serialize;

use crate::ast::Type;
use crate::decode::Operations;
use crate::facts::{
    ACC_STATIC, ACC_SYNTHETIC, CallTarget, ClassMembers, FieldAccess, InvokeKind, MemberBody,
    Operation,
};
use crate::lambda::parse_method;
use crate::pass::{ACCESSOR, IrTable, Precondition, RuleVersion};
use crate::refusal::Refusal;

/// The marker a compiler puts in front of the name of a member it generated for an access.
const MARKER: &str = "access$";

/// The pass answerable for every verdict of this module.
pub(crate) const RULE: RuleVersion = ACCESSOR.rule();

/// What one call site was decided to be.
pub(crate) enum Verdict {
    /// The call site calls a synthetic accessor whose body is one field access: present it.
    Accessor { evidence: Evidence, shape: Shape },
    /// The call site names something this rule was supposed to decide and could not: the refusal
    /// says which link failed, and the call keeps the presentation it had.
    Refused {
        evidence: Evidence,
        refusal: Refusal,
    },
    /// Not this rule's business — an ordinary call. Nothing is recorded for it.
    Ordinary,
}

/// Everything this rule read about one call site and its callee, in the vocabulary a report reads.
pub(crate) struct Evidence {
    pub(crate) owner: String,
    pub(crate) name: String,
    pub(crate) descriptor: String,
    /// The callee's access flags, when the run's member table holds the member.
    pub(crate) access_flags: Option<u16>,
    /// The field the accessor's body named, when the body was read far enough to state it.
    pub(crate) field: Option<AccessorField>,
    /// Which of the two bodies it was, when it was one of them.
    pub(crate) shape: Option<AccessorShape>,
}

/// One field access a verified accessor performed.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AccessorField {
    /// The field's owner in internal form, exactly as the class file states it.
    pub owner: String,
    /// The field's name.
    pub name: String,
    /// The field's descriptor.
    pub descriptor: String,
}

/// Which of the two verified bodies one accessor has.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessorShape {
    /// `load 0; getfield; return` — the accessor reads one instance field.
    FieldRead,
    /// `load 0; load 1; putfield; return` — the accessor writes one instance field.
    FieldWrite,
}

/// What one verified accessor call is, in the terms the builder writes.
pub(crate) struct Shape {
    /// Which of the two bodies the callee has.
    pub(crate) kind: AccessorShape,
    /// The field's name, as the callee's own decode names it.
    pub(crate) name: String,
    /// The BCI of the field access **inside the callee's body**: the second original anchor the
    /// derived presentation carries.
    pub(crate) field_bci: u32,
}

/// Decides what one call site is.
///
/// The callee's body is read from [`MemberBody`], decoded against the presented run's own pool: the
/// member is declared in the same class file, so its references resolve in the pool the payload
/// already holds.
pub(crate) fn verify(
    target: &CallTarget,
    members: Option<&ClassMembers>,
    pool: &[CpEntryFacts],
) -> Verdict {
    let name = target.name().to_string();
    if !matches!(target.kind(), InvokeKind::Static) || !name.starts_with(MARKER) {
        // An accessor is a static member a compiler generated; a call that is not both of those is
        // an ordinary call, and this rule says nothing about it.
        return Verdict::Ordinary;
    }
    let mut evidence = Evidence {
        owner: target.owner().to_string(),
        name: name.clone(),
        descriptor: target.descriptor().to_string(),
        access_flags: None,
        field: None,
        shape: None,
    };
    let Some(members) = members else {
        return Verdict::Refused {
            evidence,
            refusal: Refusal::unmet(
                &ACCESSOR,
                Precondition::IrTable(IrTable::Members),
                format!(
                    "the call site names `{}.{name}`, the marker a compiler puts in front of the member it generated for an access, and this run holds no member table of that class: the member's declaration and body are not in this run, so nothing about it is decided",
                    target.owner()
                ),
            ),
        };
    };
    if members.owner() != target.owner() {
        return Verdict::Refused {
            evidence,
            refusal: not_a_member(
                target,
                &format!(
                    "the member table this run holds is {}'s, and the call names {}.{name}",
                    members.owner(),
                    target.owner()
                ),
            ),
        };
    }
    let Some(member) = members
        .members()
        .iter()
        .find(|member| member.name() == name && member.descriptor() == target.descriptor())
    else {
        return Verdict::Refused {
            evidence,
            refusal: not_a_member(
                target,
                &format!(
                    "the class declares no `{name}{}` of its own, so this call reaches a member this run cannot read",
                    target.descriptor()
                ),
            ),
        };
    };
    evidence.access_flags = Some(member.access_flags());
    if member.access_flags() & (ACC_SYNTHETIC | ACC_STATIC) != ACC_SYNTHETIC | ACC_STATIC {
        return Verdict::Refused {
            evidence,
            refusal: Refusal::shape(
                "jre_accessor_declaration",
                format!(
                    "the member `{name}{}` is declared with the flags {:#06x}, and an accessor a compiler generated is both static and synthetic: a member a source could have written is not one this rule reads",
                    member.descriptor(),
                    member.access_flags()
                ),
            ),
        };
    }
    match forward_shape(member, members.owner(), pool, &mut evidence) {
        Ok(shape) => Verdict::Accessor { evidence, shape },
        Err(refusal) => Verdict::Refused { evidence, refusal },
    }
}

/// The refusal of a call that names something this rule cannot read as a member.
fn not_a_member(target: &CallTarget, detail: &str) -> Refusal {
    Refusal::shape(
        "jre_accessor_not_a_member",
        format!(
            "the call site at `{}.{}` was not presented as a field access: {detail}",
            target.owner(),
            target.name()
        ),
    )
}

/// Reads the callee's body, or states why it is not one field access.
fn forward_shape(
    member: &MemberBody,
    owner: &str,
    pool: &[CpEntryFacts],
    evidence: &mut Evidence,
) -> Result<Shape, Refusal> {
    let operations = Operations::of(member.code(), pool);
    let bcis: Vec<u32> = member
        .code()
        .instructions
        .iter()
        .map(|instruction| instruction.bci)
        .collect();
    let at = |index: usize| bcis.get(index).copied();
    let operation = |index: usize| at(index).and_then(|bci| operations.get(bci));
    let body = |detail: &str| {
        Refusal::shape(
            "jre_accessor_body",
            format!(
                "the body of `{}{}` is not one field access forwarded: {detail}",
                member.name(),
                member.descriptor()
            ),
        )
    };
    let Some((parameters, returns)) = parse_method(member.descriptor()) else {
        return Err(Refusal::shape(
            "jre_accessor_shape",
            format!(
                "the descriptor `{}` of `{}` is not one this layer reads",
                member.descriptor(),
                member.name()
            ),
        ));
    };
    // An accessor is given the instance it reaches, and — when it writes — the value too: one
    // parameter for a read and two for a write, which the field access below decides between.
    if parameters.is_empty() || parameters.len() > 2 {
        return Err(Refusal::shape(
            "jre_accessor_shape",
            format!(
                "the descriptor `{}` of `{}` takes {} parameter(s), and an accessor is given exactly the instance it reads or writes (and, when it writes, the value)",
                member.descriptor(),
                member.name(),
                parameters.len()
            ),
        ));
    }
    let receiver_is_the_owner = matches!(
        &parameters[0],
        Type::Reference(name) if name.as_str() == source_name(owner).as_str()
    );
    if !receiver_is_the_owner {
        return Err(Refusal::shape(
            "jre_accessor_shape",
            format!(
                "the descriptor `{}` of `{}` takes `{}` as its first parameter, and the instance an accessor is given is a `{}`",
                member.descriptor(),
                member.name(),
                parameters[0].spell(),
                source_name(owner)
            ),
        ));
    }
    // The body forwards exactly one field access: the reads of the instance (and, for a write, of
    // the value) come first, in slot order, and the access is what ends before the return.
    let Some(field_at) = bcis.iter().position(|bci| {
        matches!(
            operations.get(*bci),
            Some(Operation::Field {
                is_static: false,
                ..
            })
        )
    }) else {
        return Err(body("it holds no access to an instance field"));
    };
    for index in 0..field_at {
        let expected = u16::try_from(index).expect("a fixture's slot index fits u16");
        if operation(index) != Some(&Operation::Load { slot: expected }) {
            return Err(body(
                "the instructions before the field access are not the reads of the instance and of the value, in slot order",
            ));
        }
    }
    let Some(Operation::Field {
        access,
        is_static: false,
        owner: field_owner,
        name,
        descriptor,
    }) = operation(field_at)
    else {
        unreachable!("the position was found by the same pattern");
    };
    if field_owner != owner {
        return Err(Refusal::shape(
            "jre_accessor_field",
            format!(
                "the body of `{}{}` accesses the field `{field_owner}.{name}`, and the field an accessor reaches is its own class's",
                member.name(),
                member.descriptor()
            ),
        ));
    }
    let field_type = parse_method(&format!("(){descriptor}")).and_then(|(_, returns)| returns);
    let field_bci = at(field_at).expect("the field access's own BCI");
    let ends = bcis.len() == field_at + 2 && operation(field_at + 1) == Some(&Operation::Return);
    let kind = match access {
        FieldAccess::Read => {
            if !ends {
                return Err(body(
                    "a read accessor's body is the read of the instance, one field read and the return of what it read",
                ));
            }
            if parameters.len() != 1 {
                return Err(Refusal::shape(
                    "jre_accessor_shape",
                    format!(
                        "the body of `{}{}` reads the field `{name}` and takes {} parameter(s), and a read accessor is given the instance alone",
                        member.name(),
                        member.descriptor(),
                        parameters.len()
                    ),
                ));
            }
            if returns != field_type {
                return Err(Refusal::shape(
                    "jre_accessor_field",
                    format!(
                        "the body of `{}{}` returns `{}` and reads the field `{name}` of type `{}`",
                        member.name(),
                        member.descriptor(),
                        returns
                            .as_ref()
                            .map_or("nothing".to_string(), |ty| ty.spell().to_string()),
                        field_type
                            .as_ref()
                            .map_or("nothing".to_string(), |ty| ty.spell().to_string()),
                    ),
                ));
            }
            AccessorShape::FieldRead
        }
        FieldAccess::Write => {
            if !ends {
                return Err(body(
                    "a write accessor's body is the read of the instance, the read of the value, one field write and a return",
                ));
            }
            if returns.is_some() {
                return Err(Refusal::shape(
                    "jre_accessor_field",
                    format!(
                        "the body of `{}{}` writes the field `{name}` and returns a value, and the call site that used to spell it is a statement",
                        member.name(),
                        member.descriptor()
                    ),
                ));
            }
            if parameters.len() != 2 {
                return Err(Refusal::shape(
                    "jre_accessor_shape",
                    format!(
                        "the body of `{}{}` writes the field `{name}` and takes {} parameter(s), and a write accessor is given the instance and the value",
                        member.name(),
                        member.descriptor(),
                        parameters.len()
                    ),
                ));
            }
            if parameters.get(1) != field_type.as_ref() {
                return Err(Refusal::shape(
                    "jre_accessor_field",
                    format!(
                        "the body of `{}{}` writes the field `{name}`, and the value it is given is not that field's type",
                        member.name(),
                        member.descriptor()
                    ),
                ));
            }
            AccessorShape::FieldWrite
        }
    };
    evidence.field = Some(AccessorField {
        owner: field_owner.clone(),
        name: name.clone(),
        descriptor: descriptor.clone(),
    });
    evidence.shape = Some(kind);
    Ok(Shape {
        kind,
        name: name.clone(),
        field_bci,
    })
}

/// The Java source spelling of an internal name.
fn source_name(internal: &str) -> String {
    internal.replace('/', ".")
}

/// What one synthetic accessor call was presented as, or why it was not (P3 2.2, A12).
///
/// This is the record A12's presentation half asks for, and — like the lambda record — it exists for
/// the refusals as much as for the presentations: it states the call site (its BCI), the member the
/// call named with the flags the class declared, the field the member's own body read or wrote, and
/// which of the two bodies it was. Nothing here is re-derived from the produced text: the field's
/// name and descriptor are the ones the callee's decode stated, and the field's BCI is the anchor
/// the derived presentation carries beside the call site's.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AccessorRecord {
    /// The BCI of the call site in the presented body.
    pub call_site: u32,
    /// The owner the call named, in internal form.
    pub owner: String,
    /// The member name the call named.
    pub name: String,
    /// The member descriptor the call named.
    pub descriptor: String,
    /// The callee's access flags, when the run's member table held the member.
    pub access_flags: Option<u16>,
    /// The field the callee's body accesses, when it was read far enough to state one.
    pub field: Option<AccessorField>,
    /// Which of the two verified bodies the callee has, when it has one.
    pub shape: Option<AccessorShape>,
    /// Whether the call site was presented as a direct field access.
    pub presented: bool,
    /// Why it was not, when it was not.
    pub refusal: Option<AccessorRefusal>,
}

impl AccessorRecord {
    /// One record from what the rule read and what the build did with it.
    pub(crate) fn of(
        call_site: u32,
        evidence: &Evidence,
        presented: Option<&Shape>,
        refusal: Option<&Refusal>,
    ) -> Self {
        Self {
            call_site,
            owner: evidence.owner.clone(),
            name: evidence.name.clone(),
            descriptor: evidence.descriptor.clone(),
            access_flags: evidence.access_flags,
            field: evidence.field.clone(),
            shape: evidence.shape,
            presented: presented.is_some(),
            refusal: refusal.map(|refusal| AccessorRefusal::of(refusal, call_site)),
        }
    }

    /// Whether this call site was presented as a direct field access.
    pub fn presented(&self) -> bool {
        self.presented
    }

    /// The rule this record is answerable to: the one that presented the call or refused it.
    pub fn rule(&self) -> RuleVersion {
        RULE
    }
}

/// Why one accessor call site was not presented as a direct field access.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AccessorRefusal {
    /// The diagnostic code, in the recovery layer's own vocabulary: one `jre_accessor_*` per link of
    /// the chain that can fail.
    pub code: &'static str,
    /// The rule that refused the call.
    pub rule: RuleVersion,
    /// The declared requirement that fell short, when the refusal is one of the rule's own
    /// preconditions rather than a shape that simply is not this one.
    pub requirement: Option<String>,
    /// One sentence stating which link failed, with the call site's BCI in it.
    pub message: String,
}

impl AccessorRefusal {
    /// The refusal of one call site, as the report records it.
    pub(crate) fn of(refusal: &Refusal, call_site: u32) -> Self {
        Self {
            code: refusal.code(),
            rule: RULE,
            requirement: refusal.requirement().map(Precondition::describe),
            message: format!(
                "the call site at BCI {call_site} was not presented as a field access: {}",
                refusal.message()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_marker_decides_whether_a_call_is_worth_a_verdict_and_nothing_else() {
        // A call whose member the run does not hold is refused *with the marker named*, and an
        // ordinary call is not this rule's business at all — which is the difference between "this
        // looks like an accessor and could not be decided" and "this is not an accessor question".
        let accessor_call = CallTarget::new(InvokeKind::Static, "Test", "access$100", "(LTest;)I");
        let ordinary_call = CallTarget::new(InvokeKind::Static, "Test", "size", "()I");
        let virtual_call = CallTarget::new(InvokeKind::Virtual, "Test", "access$100", "(LTest;)I");
        assert!(matches!(
            verify(&ordinary_call, None, &[]),
            Verdict::Ordinary
        ));
        assert!(matches!(
            verify(&virtual_call, None, &[]),
            Verdict::Ordinary
        ));
        match verify(&accessor_call, None, &[]) {
            Verdict::Refused { refusal, .. } => {
                assert_eq!(refusal.code(), "jre_accessor_members_missing");
                assert_eq!(
                    refusal.requirement(),
                    Some(Precondition::IrTable(IrTable::Members))
                );
            }
            _ => panic!("a site with no member table states the table it is missing"),
        }
    }
}
