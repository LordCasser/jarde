//! The bridge method a compiler generated: which forward this layer may present, and which it must
//! leave as bytecode (P3 2.2, rule `bridge@1`).
//!
//! # What a bridge is, and what makes erasing its cast safe
//!
//! A generic or covariant override gets a second member beside it: the compiler generates one with
//! the **erased** signature that forwards to the real one, and the class declares it as a bridge
//! (`ACC_BRIDGE`, and normally `ACC_SYNTHETIC` too). Its body is the erasure in instructions:
//!
//! ```text
//! [load every parameter slot, in order] ─ invoke T.m(…) ─ [checkcast C] ─ return
//! ```
//!
//! The `checkcast` is the interesting part: a cast **is not** an erasure in general — it can fail,
//! and dropping one changes what the method does. What makes it one here is the proof that it
//! *cannot* fail: the value it casts is exactly the value the forwarded invocation returned, and
//! `C` is exactly the type that invocation's own descriptor declares it returns. A value of a
//! declared type is either null or of that type, and a cast on it can neither throw nor change it,
//! so writing the forward without the cast writes the same thing. That proof is per cast, from two
//! pool facts, and it is the only reason this rule ever drops one.
//!
//! # What the rule reads before it looks at a body
//!
//! The **declaration** comes first: whether the member is a bridge is a fact of the class's access
//! flags ([`MethodFacts::is_bridge`]), never an inference from a body that looks like a forward. A
//! member whose body is a forward and whose class did not declare it a bridge keeps its cast quoted
//! like any other operation this subset does not model, and the run records exactly that
//! (`jre_bridge_not_declared`). A run whose facts state no access flags at all cannot decide, and
//! records the declaration it lacks ([`Precondition::Metadata`] for `access_flags`) rather than
//! guessing — this is why the rule declares a metadata precondition at all.
//!
//! # What is presented, and what is refused
//!
//! Every instruction of the body must be one of the four, in that order, with the parameter slots
//! loaded exactly in order and handed to the invocation unchanged: that is "it only forwards". A
//! bridge that also stores, calls twice, computes or casts a *parameter* is not this shape; the run
//! records the refusal and presents the body the ordinary way, where the instruction it cannot
//! model is quoted. Nothing about the artifact's original facts changes: an `access$…`/bridge call
//! site is still a call in the class file, and the presentation is derived from it, not written
//! back over it.

use jarde_jvm::method_ir::{Slot, SsaInstruction, SsaTable, ValueId};
use serde::Serialize;

use crate::ast::Type;
use crate::build::stack_operands;
use crate::decode::Operations;
use crate::facts::{ACC_STATIC, InvokeKind, MethodFacts, Operation};
use crate::lambda::parse_method;
use crate::pass::{BRIDGE, Precondition, RuleVersion};
use crate::refusal::Refusal;

/// The pass answerable for every verdict of this module.
pub(crate) const RULE: RuleVersion = BRIDGE.rule();

/// What one body was decided to be, in the terms the builder reads.
pub(crate) struct Plan {
    /// The BCI of the `checkcast` this rule proved to be the erasure of the forward's return value,
    /// when the bridge has one. That instruction produces no text of its own: the value it casts is
    /// written where the forward wrote it.
    cast: Option<u32>,
    /// The verdict, as the report reads it back.
    record: BridgeRecord,
}

impl Plan {
    /// Whether this rule owns one instruction: the erased cast, and nothing else.
    pub(crate) fn owns(&self, bci: u32) -> bool {
        self.cast == Some(bci)
    }

    /// The verdict to record for this method.
    pub(crate) fn record(&self) -> &BridgeRecord {
        &self.record
    }
}

/// The verdict of one member's body, when this rule has one to state.
///
/// `Some` for a member the facts declare a bridge (the rule states whether it presented it), and
/// for a member whose body is exactly the forward-with-cast shape but whose declaration the run
/// does not hold (the rule states the declaration it is missing). `None` for every other body: an
/// ordinary member is not this rule's business, and it records nothing about it.
pub(crate) fn plan(facts: &MethodFacts, ssa: &SsaTable, operations: &Operations) -> Option<Plan> {
    let instructions: Vec<&SsaInstruction> = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .collect();
    let forward = forward_shape(facts, &instructions, operations);
    match (facts.access_flags(), facts.is_bridge()) {
        // No declaration facts: the run cannot tell a bridge from an ordinary member, so it decides
        // nothing — except for a body that *is* the forward-with-cast shape, where the honest answer
        // is the fact the run is missing rather than a presentation on a guess.
        (None, _) => {
            let forward = forward.ok().flatten()?;
            Some(Plan {
                cast: None,
                record: BridgeRecord {
                    forwarded: Some(forward.target.clone()),
                    erased: Some(forward.cast_type.clone()),
                    presented: false,
                    refusal: Some(BridgeRefusal::of(
                        &Refusal::unmet(
                            &BRIDGE,
                            Precondition::Metadata {
                                attribute: "access_flags",
                            },
                            format!(
                                "the body casts the value the invocation at BCI {} returned, and this run states no access flags for the member: whether the class declared it a bridge is not decided",
                                forward.cast_bci
                            ),
                        ),
                        forward.cast_bci,
                    )),
                },
            })
        }
        // The class declared this member and did not call it a bridge. Then the body — however much
        // it looks like a forward — keeps its cast quoted, and the run states that the declaration
        // is what kept it.
        (Some(_), false) => {
            let forward = forward.ok().flatten()?;
            Some(Plan {
                cast: None,
                record: BridgeRecord {
                    forwarded: Some(forward.target.clone()),
                    erased: Some(forward.cast_type.clone()),
                    presented: false,
                    refusal: Some(BridgeRefusal::of(
                        &Refusal::shape(
                            "jre_bridge_not_declared",
                            "the class declares this member without the bridge flag, so this rule does not erase its cast: a body that merely looks like a forward is not a bridge".to_string(),
                        ),
                        forward.cast_bci,
                    )),
                },
            })
        }
        // A declared bridge: the rule states whether it presented the forward.
        (Some(_), true) => match forward {
            Ok(Some(forward)) => Some(Plan {
                cast: Some(forward.cast_bci),
                record: BridgeRecord {
                    forwarded: Some(forward.target.clone()),
                    erased: Some(forward.cast_type.clone()),
                    presented: true,
                    refusal: None,
                },
            }),
            Ok(None) => Some(Plan {
                cast: None,
                record: BridgeRecord {
                    forwarded: None,
                    erased: None,
                    presented: true,
                    refusal: None,
                },
            }),
            Err(refusal) => Some(Plan {
                cast: None,
                record: BridgeRecord {
                    forwarded: None,
                    erased: None,
                    presented: false,
                    refusal: Some(BridgeRefusal::of(&refusal, 0)),
                },
            }),
        },
    }
}

/// One verified forward: what it calls, the cast that is the erasure, and where that cast is.
struct Forward {
    target: String,
    cast_type: String,
    cast_bci: u32,
}

/// Verifies the forward shape of one body.
///
/// `Ok(Some(forward))` is the forward *with* a redundant cast, `Ok(None)` the same forward without
/// one (which needs no erasure and no ownership: its instructions are presented by the ordinary
/// path already), and `Err` the shape this rule will not present.
fn forward_shape(
    facts: &MethodFacts,
    instructions: &[&SsaInstruction],
    operations: &Operations,
) -> Result<Option<Forward>, Refusal> {
    let descriptor = facts.descriptor();
    let Some((parameters, _)) = parse_method(descriptor) else {
        return Err(Refusal::shape(
            "jre_bridge_shape",
            format!("the member's own descriptor `{descriptor}` is not one this layer reads"),
        ));
    };
    let is_static = matches!(facts.access_flags(), Some(flags) if flags & ACC_STATIC != 0);
    // The slots the instructions load, in the order the signature declares them: an instance bridge
    // is given its receiver first, and every bridge is given its own parameters in order. A body
    // that loads anything else is not forwarding the call it declares.
    let mut slots: Vec<u16> = Vec::new();
    let mut slot = 0u16;
    if !is_static {
        slots.push(0);
        slot = 1;
    }
    for parameter in &parameters {
        slots.push(slot);
        slot += match parameter {
            Type::Long | Type::Double => 2,
            _ => 1,
        };
    }
    let mut loaded: Vec<ValueId> = Vec::new();
    let mut cursor = 0usize;
    for expected in &slots {
        let Some(instruction) = instructions.get(cursor) else {
            return Err(shape_error(
                "the body ends before it has loaded every parameter",
            ));
        };
        match operations.get(instruction.bci()) {
            Some(Operation::Load { slot }) if slot == expected => {
                if let Some((_, value)) = instruction
                    .writes()
                    .iter()
                    .find(|(slot, _)| matches!(slot, Slot::Stack(_)))
                {
                    loaded.push(*value);
                }
                cursor += 1;
            }
            _ => {
                return Err(shape_error(format!(
                    "the instruction at BCI {} is not the load of the parameter in slot {expected}",
                    instruction.bci()
                )));
            }
        }
    }
    let Some(invoke) = instructions.get(cursor) else {
        return Err(shape_error("the body ends before it forwards anything"));
    };
    // A cast *between* the bridge's own parameters and the invocation is a cast on a parameter: the
    // value it checks comes from the erased signature and the invocation's own descriptor takes a
    // narrower type, so the check can fail and this rule does not present the forward.
    if let Some(Operation::CheckCast { ty }) = operations.get(invoke.bci()) {
        return Err(Refusal::shape(
            "jre_bridge_cast_not_erasure",
            format!(
                "the bridge casts a parameter to `{ty}` at BCI {} before it forwards: a value the erased signature declares as the wider type is not proven to be the narrower one, so that cast is a check that can fail — it is not the erasure of a forwarded value",
                invoke.bci()
            ),
        ));
    }
    let Some(Operation::Invoke(target)) = operations.get(invoke.bci()) else {
        return Err(shape_error(format!(
            "the instruction at BCI {} is not the invocation a bridge forwards to",
            invoke.bci()
        )));
    };
    // The parameters are handed over unchanged: what the invocation reads off the stack is exactly
    // the values the parameter slots produced, in order (its receiver first, when it takes one).
    let operands: Vec<ValueId> = stack_operands(invoke)
        .into_iter()
        .map(|(_, value)| value)
        .collect();
    let receiver = usize::from(!matches!(target.kind(), InvokeKind::Static));
    let Some(receiver_value) = (receiver == 1).then(|| operands.first().copied()).flatten() else {
        return Err(shape_error(
            "the invocation reads no receiver, and the bridge's own signature takes one",
        ));
    };
    if receiver == 1 && !loaded.first().copied().is_some_and(|v| v == receiver_value) {
        return Err(shape_error(
            "the invocation is called on a value that is not the receiver the bridge was given",
        ));
    }
    let given: Vec<ValueId> = operands.iter().skip(receiver).copied().collect();
    let forwarded: Vec<ValueId> = loaded.iter().skip(receiver).copied().collect();
    if given != forwarded {
        return Err(shape_error(
            "the invocation is not given the bridge's own parameters, in order",
        ));
    }
    // The result the invocation produced: what the cast casts and the return returns.
    let Some(produced) = invoke
        .writes()
        .iter()
        .find(|(slot, _)| matches!(slot, Slot::Stack(_)))
        .map(|(_, value)| *value)
    else {
        return Err(shape_error(
            "the invocation a bridge forwards to returns nothing this run states",
        ));
    };
    cursor += 1;
    let Some(after) = instructions.get(cursor) else {
        return Err(shape_error(
            "the body ends before it returns the forwarded value",
        ));
    };
    let cast = match operations.get(after.bci()) {
        Some(Operation::CheckCast { ty }) => {
            let reads = after
                .reads()
                .iter()
                .find(|(slot, _)| matches!(slot, Slot::Stack(_)))
                .map(|(_, value)| *value);
            if reads != Some(produced) {
                return Err(Refusal::shape(
                    "jre_bridge_cast_not_erasure",
                    format!(
                        "the cast at BCI {} casts a value that is not the one the forwarded invocation returned, so it is a check this rule cannot prove cannot fail",
                        after.bci()
                    ),
                ));
            }
            // The cast cannot fail, and that is a fact about the **forwarded invocation's own
            // descriptor**: a value the JVM hands back as a `C` is either null or a `C`, so casting
            // it to `C` neither throws nor changes it. What the bridge declares it returns is a
            // different question (it is the *erased* type, which is why the cast exists at all), and
            // it is not evidence about the cast.
            let invocation_returns_the_cast = matches!(
                parse_method(target.descriptor()).and_then(|(_, returns)| returns),
                Some(Type::Reference(name)) if name == source_name(ty)
            );
            if !invocation_returns_the_cast {
                return Err(Refusal::shape(
                    "jre_bridge_cast_not_erasure",
                    format!(
                        "the cast at BCI {} casts to `{ty}` and the forwarded invocation `{}.{}` does not declare that as its return type, so the cast is not the erasure of the forwarded value: it is a check that can fail",
                        after.bci(),
                        target.owner(),
                        target.name()
                    ),
                ));
            }
            cursor += 1;
            Some((after, ty.clone()))
        }
        _ => None,
    };
    let Some(returned) = instructions.get(cursor) else {
        return Err(shape_error("the body ends without a return"));
    };
    if operations.get(returned.bci()) != Some(&Operation::Return) {
        return Err(shape_error(format!(
            "the instruction at BCI {} is not the return a bridge ends in",
            returned.bci()
        )));
    }
    if instructions.len() != cursor + 1 {
        return Err(shape_error(format!(
            "the body holds {} instruction(s) beyond the forward, and a bridge does nothing else",
            instructions.len() - cursor - 1
        )));
    }
    // The return returns the forwarded value (or the erased cast of it) and nothing else.
    let forwarded_value = cast
        .as_ref()
        .and_then(|(instruction, _)| {
            instruction
                .writes()
                .iter()
                .find(|(slot, _)| matches!(slot, Slot::Stack(_)))
                .map(|(_, value)| *value)
        })
        .unwrap_or(produced);
    let returned_value = returned
        .reads()
        .iter()
        .find(|(slot, _)| matches!(slot, Slot::Stack(_)))
        .map(|(_, value)| *value);
    if returned_value != Some(forwarded_value) {
        return Err(shape_error(
            "the return does not return the value the forward produced",
        ));
    }
    Ok(cast.map(|(instruction, ty)| Forward {
        target: format!(
            "{}.{}{}",
            source_name(target.owner()),
            target.name(),
            target.descriptor()
        ),
        cast_type: ty,
        cast_bci: instruction.bci(),
    }))
}

/// A shape refusal that names the forward the body failed to be.
fn shape_error(detail: impl std::fmt::Display) -> Refusal {
    Refusal::shape(
        "jre_bridge_shape",
        format!(
            "the body is not the forward a bridge writes: {detail} — a bridge's every instruction is a load of one of its own parameter slots, the single invocation it forwards to, the erasure of that invocation's return value, and the return itself"
        ),
    )
}

/// The Java source spelling of an internal name.
fn source_name(internal: &str) -> String {
    internal.replace('/', ".")
}

/// What one bridge member was presented as, or why it was not (P3 2.2).
///
/// This record is the bridge acceptance's evidence: the member the class declared as a bridge, the
/// invocation it forwarded to (with that invocation's own descriptor), the cast this rule proved to
/// be the erasure, and whether the body was presented. A bridge's presentation is *derived* — the
/// original facts of the artifact are untouched — which is why the record names the forwarded
/// member rather than claiming to have replaced it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BridgeRecord {
    /// The member the body forwards to, as `owner.name(descriptor)`, when a forward was read.
    pub forwarded: Option<String>,
    /// The erased type the bridge casts to, when it casts.
    pub erased: Option<String>,
    /// Whether the body was presented as the forward it is.
    pub presented: bool,
    /// Why it was not, when it was not.
    pub refusal: Option<BridgeRefusal>,
}

impl BridgeRecord {
    /// Whether this body was presented as the forward its declaration states.
    pub fn presented(&self) -> bool {
        self.presented
    }

    /// The rule this record is answerable to: the one that presented the body or refused it.
    pub fn rule(&self) -> RuleVersion {
        RULE
    }
}

/// Why one bridge body was not presented as a forward.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BridgeRefusal {
    /// The diagnostic code, in the recovery layer's own vocabulary: one `jre_bridge_*` per link of
    /// the shape's verification that can fail.
    pub code: &'static str,
    /// The rule that refused the body.
    pub rule: RuleVersion,
    /// The declared requirement that fell short, when the refusal is one of the rule's own
    /// preconditions rather than a shape that simply is not this one.
    pub requirement: Option<String>,
    /// One sentence stating which link failed.
    pub message: String,
}

impl BridgeRefusal {
    /// The refusal of one body, as the report records it.
    pub(crate) fn of(refusal: &Refusal, at: u32) -> Self {
        Self {
            code: refusal.code(),
            rule: RULE,
            requirement: refusal.requirement().map(Precondition::describe),
            message: format!(
                "the bridge body at BCI {at} was not presented as a forward: {}",
                refusal.message()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::facts::MethodFacts;

    #[test]
    fn only_a_declared_bridge_is_this_rules_business() {
        // The declaration, not the body, decides: a member the facts do not declare a bridge is
        // never presented through this rule, and one whose facts state nothing at all is not
        // decided at all.
        let ordinary = MethodFacts::new("get", "()Ljava/lang/String;", 1).with_access_flags(0x0001);
        assert!(!ordinary.is_bridge());
        let as_bridge = ordinary.clone().with_access_flags(0x0001 | 0x0040);
        assert!(as_bridge.is_bridge());
        assert_eq!(ordinary.access_flags(), Some(0x0001));
        assert_eq!(MethodFacts::new("get", "()V", 0).access_flags(), None);
        assert!(!MethodFacts::new("get", "()V", 0).is_bridge());
    }
}
