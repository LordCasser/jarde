//! 5.3's independent Frame expectation: a second, deliberately naive derivation of the block
//! entry states, compared against the table 4.1 publishes.
//!
//! Why this exists: the frames are what every later slice reads its locals and operand stacks
//! from, and a table that audits itself is no audit at all. The `ssa` oracle
//! ([`crate::ssa_oracle`]) recomputes the *reaching definitions* over the frames — but it takes
//! the frames themselves for granted, so a propagation mistake of the frame pass (R9: a block
//! whose exit state is stable while the locals its throw sites hand a handler are not) passes
//! through it unnoticed. This module recomputes the thing the ssa oracle assumes.
//!
//! What is independent, precisely:
//!
//! * **The transfer.** Every instruction of the subset below is applied by this module's own
//!   arms, over this module's own slot vocabulary ([`OValue`]), stack and locals model. Nothing
//!   calls the frame pass's dense opcode table, its instruction transfer, its frame and locals
//!   merges or its replay of a published state; the module's own source scan
//!   (`the_oracle_names_no_transfer_helper_of_the_frame_pass`) keeps that true.
//! * **The fixed point.** The oracle walks a plain worklist until nothing changes and has **no
//!   skip condition at all**. R9 was a skip condition (the block's exit state being equal) that
//!   was weaker than the input that had changed, so "never skip" is the direct expectation of
//!   it: the states it computes are the ones the body's own instructions imply.
//! * **The entry state.** The oracle parses the method's descriptor and access flags itself; it
//!   does not ask 4.1 which slots the caller starts values in.
//!
//! What it is **not** independent of, stated plainly:
//!
//! * The **canonical graph is an input**: the block partition, the edges, the handler rows and
//!   the throw sites come from [`CanonicalCfg`]. The oracle recomputes the *frames*, not the
//!   graph; a graph defect (as 4.2b's third fix found in the handler rows) is visible here only
//!   through the states the graph's own facts imply.
//! * The oracle covers a **declared opcode subset** ([`covered`]). A body that uses an
//!   instruction outside it is not compared at all: the comparison abstains with the reason, and
//!   the tests assert both that the covered bodies agree and that the uncovered ones abstain, so
//!   "covered" is never mistaken for "everything". `jsr`/`ret` bodies (the legacy clones) are the
//!   main family outside it, and they are covered by the canonical and clone entries of the
//!   goldens instead.
//! * Reference types are compared **by name**, and a catch-all row's caught reference is the
//!   conservative unknown both sides state. A *named* `catch_type` is outside the subset because
//!   the oracle does not read the constant pool.
//! * Category-2 locals, `Top` merging and the reference lattice follow the rules the design
//!   states (locals: incomparable ⇒ `Top`; `Null` is the bottom of the reference lattice; two
//!   named references merge to the unknown one). Where 4.1's own table states a rule differently,
//!   the comparison reports a disagreement rather than hiding it.

use std::collections::{BTreeMap, VecDeque};

use jarde_reader::classfile::{InstructionOperands, MethodCodeFacts};

use crate::canonical::{CanonicalBlockId, CanonicalCfg, CanonicalEdgeKind};
use crate::frame::{BlockFrame, FrameMethod, FrameOutcome, Value, frames};
use jarde_reader::budget::Budget;

/// The oracle's own slot vocabulary.
///
/// It is deliberately not `crate::frame::Value`: `Ref` carries only the class name the fact states
/// (`None` when the body does not establish one), and the uninitialized values of 4.2 and the
/// return addresses of the legacy dialect are absent — a body that holds one is outside the
/// subset, and the comparison says so instead of inventing a rule for it.
#[derive(Clone, Debug, Eq, PartialEq)]
enum OValue {
    /// No readable value: an unwritten local, or a slot two inputs disagree about.
    Top,
    /// The upper slot of the category-2 value held in the slot below it.
    Second,
    Int,
    Float,
    Long,
    Double,
    /// The null type: the bottom of the reference lattice.
    Null,
    /// A reference whose class name the body states, or `None` when it does not.
    Ref(Option<Vec<u8>>),
}

/// One naive state: the locals array (one entry per slot) and the operand stack, bottom first
/// (one entry per value, category 2 included).
#[derive(Clone, Debug, Eq, PartialEq, Default)]
struct OState {
    locals: Vec<OValue>,
    stack: Vec<OValue>,
}

/// Why the oracle has nothing to say about one body. Every variant is asserted by a test rather
/// than silently skipped.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Abstain {
    /// An instruction outside [`covered`].
    Instruction { bci: u32, opcode: u8 },
    /// A value 4.1 may state that this model does not carry (an uninitialized value, a return
    /// address).
    Value {
        block: CanonicalBlockId,
        slot: usize,
    },
    /// A method whose entry state the oracle does not model: a constructor's `this`.
    Declaration(String),
    /// The graph holds `jsr`/`ret` structure, which is another dialect than the oracle's.
    LegacyCalls,
    /// A named `catch_type`, which would need the constant pool.
    NamedCatch { ordinal: u32 },
    /// The body's own transfer contradicts what the oracle computed: a stack of two shapes at one
    /// block entry, a stack a return leaves behind. The frame pass reports such bodies as
    /// `Inconsistent`; the comparison is not run over them.
    Contradiction {
        block: CanonicalBlockId,
        message: String,
    },
}

impl Abstain {
    fn describe(&self) -> String {
        match self {
            Self::Instruction { bci, opcode } => {
                format!("BCI {bci} is {opcode:#04x}, outside the oracle's subset")
            }
            Self::Value { block, slot } => format!(
                "the published table states a value at {block:?} slot {slot} that the oracle's \
                 model does not carry"
            ),
            Self::Declaration(reason) => format!("the entry state is not modelled: {reason}"),
            Self::LegacyCalls => {
                "the graph holds `jsr`/`ret` structure, which is outside the oracle's subset"
                    .to_string()
            }
            Self::NamedCatch { ordinal } => {
                format!("record {ordinal} names a catch type, which would need the constant pool")
            }
            Self::Contradiction { block, message } => {
                format!("{block:?}: {message}")
            }
        }
    }
}

/// The opcode families the oracle implements, listed once so the boundary is readable.
///
/// Constants, the local loads and stores (the `_<n>` and `wide` forms included), `pop`/`pop2`, the
/// whole `dup` family, `swap`, the int arithmetic family, `iinc`, the comparisons and their
/// branches, `goto`, both switch forms, the six returns, `nop` and `athrow`. Everything else —
/// `ldc` and the constant-pool readers, the array and field accesses, the invocations, `new` and
/// the uninitialized values, `checkcast`/`instanceof`, the monitor pair, the legacy `jsr`/`ret`
/// family — abstains.
fn covered(opcode: u8) -> bool {
    matches!(
        opcode,
        0x00 // nop
        | 0x01 // aconst_null
        | 0x02..=0x0f // iconst_*, lconst_*, fconst_*, dconst_*
        | 0x10 | 0x11 // bipush, sipush
        | 0x15..=0x35 // the local loads, the `_<n>` forms included
        | 0x36..=0x4e // the local stores, the `_<n>` forms included
        | 0x57..=0x5f // pop, pop2, the dup family, swap
        | 0x60 | 0x64 | 0x68 | 0x6c | 0x70 // iadd, isub, imul, idiv, irem
        | 0x74 // ineg
        | 0x78 | 0x7a | 0x7c | 0x7e | 0x80 | 0x82 // ishl, ishr, iushr, iand, ior, ixor
        | 0x84 // iinc
        | 0x99..=0xa6 // the comparisons and their branches
        | 0xc6 | 0xc7 // ifnull, ifnonnull
        | 0xa7 // goto
        | 0xaa | 0xab // tableswitch, lookupswitch
        | 0xac..=0xb1 // the returns and `return`
        | 0xbf // athrow
    )
}

/// Whether one covered instruction may raise. The subset's raisers are the two int divisions and
/// `athrow`; every other exception source (array and field access, invocations, `new`, the
/// monitors) is outside the subset already.
fn may_raise(opcode: u8) -> bool {
    matches!(opcode, 0x6c | 0x70 | 0xbf)
}

/// The class one local load or store moves, from the instruction's own opcode.
fn local_ty(opcode: u8) -> Option<OValue> {
    Some(match opcode {
        0x15 | 0x1a..=0x1d | 0x36 | 0x3b..=0x3e => OValue::Int,
        0x16 | 0x1e..=0x21 | 0x37 | 0x3f..=0x42 => OValue::Long,
        0x17 | 0x22..=0x25 | 0x38 | 0x43..=0x46 => OValue::Float,
        0x18 | 0x26..=0x29 | 0x39 | 0x47..=0x4a => OValue::Double,
        0x19 | 0x2a..=0x2d | 0x3a | 0x4b..=0x4e => OValue::Ref(None),
        _ => return None,
    })
}

/// The class one return instruction hands back.
fn return_ty(opcode: u8) -> OValue {
    match opcode {
        0xac => OValue::Int,
        0xad => OValue::Long,
        0xae => OValue::Float,
        0xaf => OValue::Double,
        0xb0 => OValue::Ref(None),
        other => panic!("the oracle does not know a return {other:#04x}"),
    }
}

/// The local slot an instruction names, from its typed operands.
fn local_slot(operands: &InstructionOperands, bci: u32) -> Result<usize, Abstain> {
    let Some(local) = operands.local else {
        return Err(Abstain::Contradiction {
            block: CanonicalBlockId {
                bci,
                path: Vec::new(),
            },
            message: "a local instruction whose operands name no local".to_string(),
        });
    };
    Ok(usize::from(local.index))
}

/// The merge of two values at one slot: `None` when the two cannot be read as one value on a
/// stack, `Top` for a local the two inputs disagree about.
///
/// The rules are the ones the design states: equal values stay, `Top` poisons a local, `Null` is
/// the bottom of the reference lattice, two named references merge to the unknown one, and any
/// other pair of classes has no common value.
fn merged(left: &OValue, right: &OValue, locals: bool) -> Option<OValue> {
    if left == right {
        return Some(left.clone());
    }
    match (left, right) {
        (OValue::Null, OValue::Ref(name)) | (OValue::Ref(name), OValue::Null) => {
            Some(OValue::Ref(name.clone()))
        }
        (OValue::Ref(_), OValue::Ref(_)) => Some(OValue::Ref(None)),
        _ => {
            if locals {
                Some(OValue::Top)
            } else {
                None
            }
        }
    }
}

/// Whether one value occupies two slots: the class a category-2 value is not.
fn is_category_two(value: &OValue) -> bool {
    matches!(value, OValue::Long | OValue::Double)
}

/// The `pop`, `pop2`, `dup` family and `swap`, over the oracle's own stack model.
fn stack_form(state: &mut OState, opcode: u8, bci: u32) -> Result<(), Abstain> {
    let short = |state: &OState| {
        format!(
            "the stack holds {} values where {opcode:#04x} needs more",
            state.stack.len()
        )
    };
    let pop = |state: &mut OState| -> Result<OValue, Abstain> {
        state.stack.pop().ok_or_else(|| Abstain::Contradiction {
            block: CanonicalBlockId {
                bci,
                path: Vec::new(),
            },
            message: "a stack operation pops from an empty stack".to_string(),
        })
    };
    match opcode {
        0x57 => {
            pop(state)?;
        }
        0x58 => {
            let first = pop(state)?;
            if !is_category_two(&first) {
                let _ = short(state);
                pop(state)?;
            }
        }
        0x59 => {
            let value = pop(state)?;
            if is_category_two(&value) {
                return Err(Abstain::Contradiction {
                    block: CanonicalBlockId {
                        bci,
                        path: Vec::new(),
                    },
                    message: "`dup` over a category-2 value".to_string(),
                });
            }
            state.stack.push(value.clone());
            state.stack.push(value);
        }
        0x5a => {
            let value1 = pop(state)?;
            let value2 = pop(state)?;
            if is_category_two(&value1) || is_category_two(&value2) {
                return Err(Abstain::Contradiction {
                    block: CanonicalBlockId {
                        bci,
                        path: Vec::new(),
                    },
                    message: "`dup_x1` over a category-2 value".to_string(),
                });
            }
            state.stack.push(value1.clone());
            state.stack.push(value2);
            state.stack.push(value1);
        }
        0x5b => {
            let value1 = pop(state)?;
            if is_category_two(&value1) {
                return Err(Abstain::Contradiction {
                    block: CanonicalBlockId {
                        bci,
                        path: Vec::new(),
                    },
                    message: "`dup_x2` over a category-2 value".to_string(),
                });
            }
            let value2 = pop(state)?;
            if is_category_two(&value2) {
                state.stack.push(value1.clone());
                state.stack.push(value2);
                state.stack.push(value1);
            } else {
                let value3 = pop(state)?;
                state.stack.push(value1.clone());
                state.stack.push(value3);
                state.stack.push(value2);
                state.stack.push(value1);
            }
        }
        0x5c => {
            let value1 = pop(state)?;
            if is_category_two(&value1) {
                state.stack.push(value1.clone());
                state.stack.push(value1);
            } else {
                let value2 = pop(state)?;
                if is_category_two(&value2) {
                    return Err(Abstain::Contradiction {
                        block: CanonicalBlockId {
                            bci,
                            path: Vec::new(),
                        },
                        message: "`dup2` over a category-2 value below a category-1 one"
                            .to_string(),
                    });
                }
                state.stack.push(value2.clone());
                state.stack.push(value1.clone());
                state.stack.push(value2);
                state.stack.push(value1);
            }
        }
        0x5d => {
            let value1 = pop(state)?;
            let value2 = pop(state)?;
            if is_category_two(&value1) || is_category_two(&value2) {
                return Err(Abstain::Contradiction {
                    block: CanonicalBlockId {
                        bci,
                        path: Vec::new(),
                    },
                    message: "`dup2_x1` over a category-2 value".to_string(),
                });
            }
            let value3 = pop(state)?;
            state.stack.push(value2.clone());
            state.stack.push(value1.clone());
            state.stack.push(value3);
            state.stack.push(value2);
            state.stack.push(value1);
        }
        0x5e => {
            let value1 = pop(state)?;
            if is_category_two(&value1) {
                let value2 = pop(state)?;
                if !is_category_two(&value2) {
                    return Err(Abstain::Contradiction {
                        block: CanonicalBlockId {
                            bci,
                            path: Vec::new(),
                        },
                        message: "`dup2_x2` form four needs two category-2 values".to_string(),
                    });
                }
                state.stack.push(value1.clone());
                state.stack.push(value2);
                state.stack.push(value1);
            } else {
                let value2 = pop(state)?;
                let value3 = pop(state)?;
                if is_category_two(&value3) {
                    state.stack.push(value2.clone());
                    state.stack.push(value1.clone());
                    state.stack.push(value3);
                    state.stack.push(value2);
                    state.stack.push(value1);
                } else {
                    let value4 = pop(state)?;
                    state.stack.push(value2.clone());
                    state.stack.push(value1.clone());
                    state.stack.push(value4);
                    state.stack.push(value3);
                    state.stack.push(value2);
                    state.stack.push(value1);
                }
            }
        }
        0x5f => {
            let value1 = pop(state)?;
            let value2 = pop(state)?;
            if is_category_two(&value1) || is_category_two(&value2) {
                return Err(Abstain::Contradiction {
                    block: CanonicalBlockId {
                        bci,
                        path: Vec::new(),
                    },
                    message: "`swap` over a category-2 value".to_string(),
                });
            }
            state.stack.push(value1);
            state.stack.push(value2);
        }
        other => panic!("the oracle does not know a stack form {other:#04x}"),
    }
    Ok(())
}

/// Projects a published value into the oracle's vocabulary.
///
/// `None` for a value this model does not carry: the comparison then abstains rather than
/// guessing.
fn project(value: &Value) -> Option<OValue> {
    Some(match value {
        Value::Top => OValue::Top,
        Value::Second => OValue::Second,
        Value::Int => OValue::Int,
        Value::Float => OValue::Float,
        Value::Long => OValue::Long,
        Value::Double => OValue::Double,
        Value::Null => OValue::Null,
        Value::Ref(crate::frame::RefType::Named { name, .. }) => {
            OValue::Ref(Some(internal_name(name)))
        }
        Value::Ref(crate::frame::RefType::Unknown) => OValue::Ref(None),
        Value::UninitializedThis | Value::Uninitialized { .. } | Value::ReturnAddress => {
            return None;
        }
    })
}

/// The internal name of a reference the published table spells either way.
///
/// The table spells a reference *parameter* as the field-descriptor slice it came from
/// (`Ljava/lang/Object;`, `[I`) and a receiver as the internal name of its class (`Test`). The
/// oracle's model keeps one spelling — the internal name — and normalizes the published one here,
/// so the comparison is about the type a slot holds and not about which of the two spellings
/// reached it. An array type is already an internal name (`[I`).
fn internal_name(name: &[u8]) -> Vec<u8> {
    if name.first() == Some(&b'L') && name.last() == Some(&b';') && name.len() >= 2 {
        name[1..name.len() - 1].to_vec()
    } else {
        name.to_vec()
    }
}

/// The oracle's own descriptor parser: the parameter types, in order.
fn parse_descriptor(descriptor: &[u8]) -> Result<Vec<OValue>, String> {
    let mut index = 0;
    let mut params = Vec::new();
    if descriptor.first() != Some(&b'(') {
        return Err("a descriptor that does not start with `(`".to_string());
    }
    index += 1;
    while descriptor.get(index).is_some_and(|byte| *byte != b')') {
        let (value, size) = parse_field_type(descriptor, index)?;
        params.push(value);
        index += size;
    }
    if descriptor.get(index) != Some(&b')') {
        return Err("a descriptor with no `)` after its parameters".to_string());
    }
    index += 1;
    let size = if descriptor.get(index) == Some(&b'V') {
        1
    } else {
        parse_field_type(descriptor, index)?.1
    };
    if index + size != descriptor.len() {
        return Err("a descriptor with bytes after its return type".to_string());
    }
    Ok(params)
}

/// One field type: the oracle's value for it and the bytes it consumes.
fn parse_field_type(descriptor: &[u8], index: usize) -> Result<(OValue, usize), String> {
    let byte = *descriptor
        .get(index)
        .ok_or_else(|| "a descriptor that ends inside a field type".to_string())?;
    Ok(match byte {
        b'B' | b'C' | b'I' | b'S' | b'Z' => (OValue::Int, 1),
        b'F' => (OValue::Float, 1),
        b'J' => (OValue::Long, 1),
        b'D' => (OValue::Double, 1),
        b'L' => {
            let rest = &descriptor[index..];
            let end = rest
                .iter()
                .position(|byte| *byte == b';')
                .ok_or_else(|| "an unterminated class type".to_string())?;
            (OValue::Ref(Some(rest[1..end].to_vec())), end + 1)
        }
        b'[' => {
            let mut end = index;
            while descriptor.get(end) == Some(&b'[') {
                end += 1;
            }
            let (_, size) = parse_field_type(descriptor, end)?;
            (
                OValue::Ref(Some(descriptor[index..end + size].to_vec())),
                end + size - index,
            )
        }
        other => return Err(format!("an unreadable field type byte {other:#04x}")),
    })
}

/// One block, in the oracle's own terms.
struct OBlock {
    id: CanonicalBlockId,
    /// Instruction indices of this block, in execution order.
    instructions: Vec<usize>,
    /// Plain successors, by block position.
    normal: Vec<usize>,
    /// Exception successors: the target, the record's ordinal, and the BCIs of this block's throw
    /// sites the record covers.
    exceptional: Vec<(usize, u32, Vec<u32>)>,
    /// The BCIs of this block's instructions that may raise, ascending.
    sites: Vec<u32>,
}

/// The oracle's own view of one body: the instructions, the block partition the graph states, and
/// the declaration facts the entry state comes from.
struct Oracle {
    /// `(BCI, effective opcode)` of every instruction, in decode order.
    instructions: Vec<(u32, u8)>,
    /// Typed operands of every instruction, in the same order.
    operands: Vec<InstructionOperands>,
    blocks: Vec<OBlock>,
    index_of: BTreeMap<CanonicalBlockId, usize>,
    access_flags: u16,
    name: Vec<u8>,
    descriptor: Vec<u8>,
    owner: Vec<u8>,
    max_locals: usize,
}

/// The result of comparing one body's frames with the oracle's expectation.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Comparison {
    /// The oracle has no expectation for this body, and why.
    Abstained(Abstain),
    /// Every slot of every block entry agrees.
    Agrees { blocks: usize },
    /// The published table states something the body's own instructions do not imply.
    Disagrees(Vec<String>),
    /// The oracle's own transfer contradicts itself over this body.
    Contradictory(String),
}

/// The deliberate weakenings the falsification tests run the oracle under. Each one is the shape
/// a plausible mistake takes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Flaws {
    /// An exception input contributes the source block's **exit** state instead of the state its
    /// named throw site was in — exactly R9's confusion, one layer away from the skip condition.
    aggregate_exception_inputs: bool,
    /// Every block is transferred once, in the order the worklist first reached it: a fixed point
    /// that stops early, which is what a body with a back edge catches.
    one_pass: bool,
    /// The comparison reads one local slot less than the published table states, as a comparator
    /// that mis-parses a slot would.
    drop_a_local: Option<usize>,
    /// The comparison trusts the published table: it answers "agrees" without looking. This is the
    /// weakening that would turn the whole module into a restatement of the pass, and the tests
    /// that plant a wrong value in a table are what catch it.
    trust_the_published_table: bool,
}

impl Oracle {
    /// Reads one body's declaration, instructions, blocks, edges and throw sites.
    fn build(
        facts: &MethodCodeFacts,
        canonical: &CanonicalCfg,
        method: &FrameMethod<'_>,
    ) -> Result<Self, Abstain> {
        let mut index_of: BTreeMap<CanonicalBlockId, usize> = BTreeMap::new();
        for (position, block) in canonical.blocks.iter().enumerate() {
            index_of.insert(block.id.clone(), position);
        }
        let instructions: Vec<(u32, u8)> = facts
            .instructions
            .iter()
            .zip(facts.operands())
            .map(|(instruction, operands)| (instruction.bci, operands.effective_opcode))
            .collect();
        let operands: Vec<InstructionOperands> = facts.operands().to_vec();
        let mut blocks = Vec::new();
        for block in &canonical.blocks {
            let start = *block
                .blocks
                .first()
                .expect("a canonical block stands for at least one original block");
            let mut held = Vec::new();
            for (index, (bci, _)) in instructions.iter().enumerate() {
                if *bci >= start && *bci < block.end_bci {
                    held.push(index);
                }
            }
            let mut sites = Vec::new();
            for index in &held {
                if may_raise(instructions[*index].1) {
                    sites.push(instructions[*index].0);
                }
            }
            blocks.push(OBlock {
                id: block.id.clone(),
                instructions: held,
                normal: Vec::new(),
                exceptional: Vec::new(),
                sites,
            });
        }
        for edge in &canonical.edges {
            let (Some(&from), Some(&to)) = (index_of.get(&edge.from), index_of.get(&edge.to))
            else {
                return Err(Abstain::Contradiction {
                    block: edge.from.clone(),
                    message: "the graph holds an edge naming a block it does not hold".to_string(),
                });
            };
            match edge.kind {
                CanonicalEdgeKind::Normal => blocks[from].normal.push(to),
                CanonicalEdgeKind::Exception { handler_ordinal } => {
                    let row = canonical
                        .handler_rows
                        .iter()
                        .find(|row| {
                            row.ordinal == handler_ordinal && row.handler.as_ref() == Some(&edge.to)
                        })
                        .ok_or(Abstain::NamedCatch {
                            ordinal: handler_ordinal,
                        })?;
                    if row.catch_type_index.is_some() {
                        return Err(Abstain::NamedCatch {
                            ordinal: handler_ordinal,
                        });
                    }
                    let sites: Vec<u32> = canonical
                        .throw_sites
                        .iter()
                        .filter(|site| {
                            site.block == edge.from && site.handlers.contains(&handler_ordinal)
                        })
                        .map(|site| site.bci)
                        .collect();
                    if sites.is_empty() {
                        return Err(Abstain::Contradiction {
                            block: edge.from.clone(),
                            message: format!(
                                "the graph holds an exception edge for record {handler_ordinal} \
                                 whose source block names no throw site"
                            ),
                        });
                    }
                    blocks[from].exceptional.push((to, handler_ordinal, sites));
                }
                CanonicalEdgeKind::Call { .. } | CanonicalEdgeKind::Return { .. } => {
                    return Err(Abstain::LegacyCalls);
                }
            }
        }
        Ok(Self {
            instructions,
            operands,
            blocks,
            index_of,
            access_flags: method.access_flags,
            name: method.name.to_vec(),
            descriptor: method.descriptor.to_vec(),
            owner: method.owner.to_vec(),
            max_locals: usize::from(facts.max_locals),
        })
    }

    /// The state the caller hands the entry block, from the declaration alone.
    fn entry_state(&self) -> Result<OState, Abstain> {
        let params = parse_descriptor(&self.descriptor).map_err(Abstain::Declaration)?;
        let mut locals = vec![OValue::Top; self.max_locals];
        let mut slot = 0usize;
        if self.access_flags & 0x0008 == 0 {
            if self.name == b"<init>" {
                return Err(Abstain::Declaration(
                    "a constructor's `this` is an uninitialized value this model does not carry"
                        .to_string(),
                ));
            }
            if self.max_locals == 0 {
                return Err(Abstain::Declaration(
                    "an instance method with no slot for `this`".to_string(),
                ));
            }
            locals[0] = OValue::Ref(Some(self.owner.clone()));
            slot = 1;
        }
        for param in params {
            let slots = if is_category_two(&param) { 2 } else { 1 };
            if slot + slots > locals.len() {
                return Err(Abstain::Declaration(
                    "a descriptor whose parameters do not fit the declared locals".to_string(),
                ));
            }
            locals[slot] = param;
            if slots == 2 {
                locals[slot + 1] = OValue::Second;
            }
            slot += slots;
        }
        Ok(OState {
            locals,
            stack: Vec::new(),
        })
    }

    /// The block one instruction belongs to, for a message.
    fn block_of(&self, index: usize) -> &CanonicalBlockId {
        self.blocks
            .iter()
            .find(|block| block.instructions.contains(&index))
            .map(|block| &block.id)
            .unwrap_or_else(|| panic!("instruction {index} is in no block"))
    }

    /// Applies one block's instructions to its entry state.
    fn transfer(&self, position: usize, entry: &OState) -> Result<OState, Abstain> {
        let mut state = entry.clone();
        for index in &self.blocks[position].instructions {
            state = self.apply(*index, state)?;
        }
        Ok(state)
    }

    /// Applies one instruction.
    fn apply(&self, index: usize, mut state: OState) -> Result<OState, Abstain> {
        let (bci, opcode) = self.instructions[index];
        if !covered(opcode) {
            return Err(Abstain::Instruction { bci, opcode });
        }
        match opcode {
            0x00 => {}
            0x01 => state.stack.push(OValue::Null),
            0x02..=0x08 => state.stack.push(OValue::Int),
            0x09 | 0x0a => state.stack.push(OValue::Long),
            0x0b..=0x0d => state.stack.push(OValue::Float),
            0x0e | 0x0f => state.stack.push(OValue::Double),
            0x10 | 0x11 => state.stack.push(OValue::Int),
            0x15..=0x35 => {
                let expected = local_ty(opcode).expect("a local load names its class");
                let value = self.read_local(&state, index, bci, expected)?;
                state.stack.push(value);
            }
            0x36..=0x4e => {
                let expected = local_ty(opcode).expect("a local store names its class");
                let value = self.pop(&mut state, &expected, bci)?;
                self.write_local(&mut state, index, bci, value)?;
            }
            0x57..=0x5f => stack_form(&mut state, opcode, bci)?,
            0x60 | 0x64 | 0x68 | 0x6c | 0x70 | 0x78 | 0x7a | 0x7c | 0x7e | 0x80 | 0x82 => {
                let _right = self.pop(&mut state, &OValue::Int, bci)?;
                let _left = self.pop(&mut state, &OValue::Int, bci)?;
                state.stack.push(OValue::Int);
            }
            0x74 => {
                let _value = self.pop(&mut state, &OValue::Int, bci)?;
                state.stack.push(OValue::Int);
            }
            0x84 => {
                let slot = local_slot(&self.operands[index], bci)?;
                match state.locals.get(slot) {
                    Some(OValue::Int) => {}
                    other => {
                        return Err(Abstain::Contradiction {
                            block: self.block_of(index).clone(),
                            message: format!(
                                "`iinc` at BCI {bci} reads local {slot}, which is {other:?} and not \
                                 an int"
                            ),
                        });
                    }
                }
            }
            0x99..=0x9e => {
                let _condition = self.pop(&mut state, &OValue::Int, bci)?;
            }
            0x9f..=0xa4 => {
                let _right = self.pop(&mut state, &OValue::Int, bci)?;
                let _left = self.pop(&mut state, &OValue::Int, bci)?;
            }
            0xa5 | 0xa6 => {
                let _right = self.pop(&mut state, &OValue::Ref(None), bci)?;
                let _left = self.pop(&mut state, &OValue::Ref(None), bci)?;
            }
            0xa7 => {}
            0xc6 | 0xc7 => {
                let _reference = self.pop(&mut state, &OValue::Ref(None), bci)?;
            }
            0xaa | 0xab => {
                let _key = self.pop(&mut state, &OValue::Int, bci)?;
            }
            0xac..=0xb0 => {
                let _value = self.pop(&mut state, &return_ty(opcode), bci)?;
            }
            0xb1 => {}
            0xbf => {
                let _thrown = self.pop(&mut state, &OValue::Ref(None), bci)?;
            }
            other => return Err(Abstain::Instruction { bci, opcode: other }),
        }
        Ok(state)
    }

    /// Pops one value, requiring the class the instruction needs (`Null` counts where a reference
    /// is needed, exactly as it does in the JVM's own type rules).
    fn pop(&self, state: &mut OState, expected: &OValue, bci: u32) -> Result<OValue, Abstain> {
        let contradiction = |message: String| Abstain::Contradiction {
            block: CanonicalBlockId {
                bci,
                path: Vec::new(),
            },
            message,
        };
        let value = state
            .stack
            .pop()
            .ok_or_else(|| contradiction("an instruction pops from an empty stack".to_string()))?;
        let fits = match expected {
            OValue::Ref(_) => matches!(value, OValue::Ref(_) | OValue::Null),
            other => &value == other,
        };
        if !fits {
            return Err(contradiction(format!(
                "an instruction at BCI {bci} needs {expected:?} and holds {value:?}"
            )));
        }
        Ok(value)
    }

    /// Reads one local, requiring the class the instruction loads.
    fn read_local(
        &self,
        state: &OState,
        index: usize,
        bci: u32,
        expected: OValue,
    ) -> Result<OValue, Abstain> {
        let slot = local_slot(&self.operands[index], bci)?;
        let value = state.locals.get(slot).cloned().unwrap_or(OValue::Top);
        let fits = match (&expected, &value) {
            (OValue::Ref(_), OValue::Ref(_) | OValue::Null) => true,
            (left, right) => left == right,
        };
        if !fits {
            return Err(Abstain::Contradiction {
                block: self.block_of(index).clone(),
                message: format!(
                    "a load at BCI {bci} needs {expected:?} in local {slot}, which is {value:?}"
                ),
            });
        }
        Ok(value)
    }

    /// Writes one value into the local the instruction names, invalidating a category-2 pair it
    /// lands on or in.
    fn write_local(
        &self,
        state: &mut OState,
        index: usize,
        bci: u32,
        value: OValue,
    ) -> Result<(), Abstain> {
        let slot = local_slot(&self.operands[index], bci)?;
        if slot >= state.locals.len() {
            return Err(Abstain::Contradiction {
                block: self.block_of(index).clone(),
                message: format!("a store at BCI {bci} writes local {slot}, past the declaration"),
            });
        }
        let two = is_category_two(&value);
        // A write over either half of a category-2 pair invalidates the other half.
        if slot > 0 && state.locals[slot - 1] == OValue::Second {
            state.locals[slot - 1] = OValue::Top;
        }
        if state.locals[slot] == OValue::Second && slot > 0 {
            state.locals[slot - 1] = OValue::Top;
        }
        state.locals[slot] = value;
        if two && slot + 1 < state.locals.len() {
            state.locals[slot + 1] = OValue::Second;
        }
        // A category-1 write into the lower half of a pair clears the upper half.
        if !two && slot + 1 < state.locals.len() && state.locals[slot + 1] == OValue::Second {
            state.locals[slot + 1] = OValue::Top;
        }
        Ok(())
    }

    /// The locals one throw site was entered with: the entry state with every instruction before
    /// it applied, and the instruction itself not applied.
    fn snapshot_before(
        &self,
        position: usize,
        entry: &OState,
        bci: u32,
    ) -> Result<OState, Abstain> {
        let mut state = entry.clone();
        for index in &self.blocks[position].instructions {
            if self.instructions[*index].0 == bci {
                return Ok(state);
            }
            state = self.apply(*index, state)?;
        }
        Err(Abstain::Contradiction {
            block: self.blocks[position].id.clone(),
            message: format!("the block holds no throw site at BCI {bci}"),
        })
    }

    /// Every state one block hands its successors: its own exit for the plain edges, and the state
    /// each of its throw sites was in for the exception edges.
    fn outgoing(
        &self,
        position: usize,
        entry: &OState,
        exit: &OState,
        flaws: &Flaws,
    ) -> Result<Vec<(usize, OState)>, Abstain> {
        let block = &self.blocks[position];
        let mut outgoing = Vec::new();
        for successor in &block.normal {
            outgoing.push((*successor, exit.clone()));
        }
        for (successor, _ordinal, sites) in &block.exceptional {
            for bci in sites {
                let mut state = if flaws.aggregate_exception_inputs {
                    exit.clone()
                } else {
                    self.snapshot_before(position, entry, *bci)?
                };
                state.stack = vec![OValue::Ref(None)];
                outgoing.push((*successor, state));
            }
        }
        Ok(outgoing)
    }

    /// Merges one incoming state into a block's entry state, answering whether anything changed.
    fn merge(
        &self,
        position: usize,
        slot: &mut Option<OState>,
        incoming: &OState,
    ) -> Result<bool, Abstain> {
        let Some(current) = slot.as_mut() else {
            *slot = Some(incoming.clone());
            return Ok(true);
        };
        if current.stack.len() != incoming.stack.len() {
            return Err(Abstain::Contradiction {
                block: self.blocks[position].id.clone(),
                message: format!(
                    "two inputs hand this block stacks of {} and {} values",
                    current.stack.len(),
                    incoming.stack.len()
                ),
            });
        }
        let before = current.clone();
        for (depth, (left, right)) in current
            .stack
            .iter_mut()
            .zip(incoming.stack.iter())
            .enumerate()
        {
            *left = merged(left, right, false).ok_or_else(|| Abstain::Contradiction {
                block: self.blocks[position].id.clone(),
                message: format!(
                    "two inputs disagree about stack slot {depth}: {left:?} and {right:?}"
                ),
            })?;
        }
        for index in 0..current.locals.len() {
            let left = current.locals[index].clone();
            let right = incoming.locals[index].clone();
            current.locals[index] = merged(&left, &right, true).unwrap_or(OValue::Top);
        }
        // A pair that stopped being one is not left half-formed: a category-2 value without its
        // `Second` (or the other way round) is no value at all.
        for index in 0..current.locals.len() {
            if is_category_two(&current.locals[index]) {
                let upper_ok = current
                    .locals
                    .get(index + 1)
                    .is_some_and(|upper| *upper == OValue::Second);
                if !upper_ok {
                    current.locals[index] = OValue::Top;
                }
            }
            if current.locals[index] == OValue::Second {
                let lower_ok = index > 0 && is_category_two(&current.locals[index - 1]);
                if !lower_ok {
                    current.locals[index] = OValue::Top;
                }
            }
        }
        Ok(*current != before)
    }

    /// The fixed point: a plain worklist over every successor, with no skip condition.
    fn fixed_point(&self, flaws: &Flaws) -> Result<Vec<Option<OState>>, Abstain> {
        let entry = CanonicalBlockId {
            bci: 0,
            path: Vec::new(),
        };
        let Some(&entry) = self.index_of.get(&entry) else {
            return Err(Abstain::Contradiction {
                block: entry,
                message: "the graph holds no entry block for a body that starts at BCI 0"
                    .to_string(),
            });
        };
        let mut entries: Vec<Option<OState>> = vec![None; self.blocks.len()];
        entries[entry] = Some(self.entry_state()?);
        let mut queue = VecDeque::from([entry]);
        let mut seen: Vec<bool> = vec![false; self.blocks.len()];
        let mut steps = 0usize;
        while let Some(position) = queue.pop_front() {
            steps += 1;
            assert!(
                steps < 100_000,
                "the oracle's fixed point does not converge: that is a defect of the oracle or of \
                 the merge it applies, not a property of a JVM body"
            );
            if flaws.one_pass && seen[position] {
                continue;
            }
            seen[position] = true;
            let entry_state = entries[position]
                .as_ref()
                .expect("a block is queued only after it has a state")
                .clone();
            let exit = self.transfer(position, &entry_state)?;
            for (successor, incoming) in self.outgoing(position, &entry_state, &exit, flaws)? {
                if self.merge(successor, &mut entries[successor], &incoming)? {
                    queue.push_back(successor);
                }
            }
        }
        Ok(entries)
    }
}

/// Compares one body's published frames with the oracle's own expectation.
fn compare(
    facts: &MethodCodeFacts,
    canonical: &CanonicalCfg,
    method: &FrameMethod<'_>,
    published: &[BlockFrame],
    flaws: &Flaws,
) -> Comparison {
    if flaws.trust_the_published_table {
        return Comparison::Agrees {
            blocks: published.len(),
        };
    }
    let oracle = match Oracle::build(facts, canonical, method) {
        Ok(oracle) => oracle,
        Err(abstain) => return Comparison::Abstained(abstain),
    };
    let entries = match oracle.fixed_point(flaws) {
        Ok(entries) => entries,
        Err(abstain @ Abstain::Contradiction { .. }) => {
            return Comparison::Contradictory(abstain.describe());
        }
        Err(abstain) => return Comparison::Abstained(abstain),
    };
    let mut mismatches = Vec::new();
    for frame in published {
        let Some(&position) = oracle.index_of.get(&frame.block) else {
            mismatches.push(format!(
                "{:?}: the published table holds a block the canonical graph does not",
                frame.block
            ));
            continue;
        };
        let Some(oracle_state) = entries[position].as_ref() else {
            mismatches.push(format!(
                "{:?}: the published table states an entry state the oracle's fixed point never \
                 reached",
                frame.block
            ));
            continue;
        };
        for (slot, (published_value, oracle_value)) in frame
            .locals
            .iter()
            .zip(oracle_state.locals.iter())
            .enumerate()
        {
            if flaws.drop_a_local == Some(slot) {
                continue;
            }
            let Some(projected) = project(published_value) else {
                return Comparison::Abstained(Abstain::Value {
                    block: frame.block.clone(),
                    slot,
                });
            };
            if &projected != oracle_value {
                mismatches.push(format!(
                    "{:?} local slot {slot}: published {:?}, oracle {:?}",
                    frame.block, projected, oracle_value
                ));
            }
        }
        if frame.locals.len() != oracle_state.locals.len() {
            mismatches.push(format!(
                "{:?}: the published state has {} local slots, the oracle {}",
                frame.block,
                frame.locals.len(),
                oracle_state.locals.len()
            ));
        }
        if frame.stack.len() != oracle_state.stack.len() {
            mismatches.push(format!(
                "{:?}: the published entry stack holds {} values, the oracle {}",
                frame.block,
                frame.stack.len(),
                oracle_state.stack.len()
            ));
        } else {
            for (depth, (published_value, oracle_value)) in frame
                .stack
                .iter()
                .zip(oracle_state.stack.iter())
                .enumerate()
            {
                let Some(projected) = project(published_value) else {
                    return Comparison::Abstained(Abstain::Value {
                        block: frame.block.clone(),
                        slot: depth,
                    });
                };
                if &projected != oracle_value {
                    mismatches.push(format!(
                        "{:?} stack slot {depth}: published {:?}, oracle {:?}",
                        frame.block, projected, oracle_value
                    ));
                }
            }
        }
    }
    for (position, state) in entries.iter().enumerate() {
        if state.is_some()
            && !published
                .iter()
                .any(|frame| frame.block == oracle.blocks[position].id)
        {
            mismatches.push(format!(
                "{:?}: the oracle reaches this block and the published table does not state it",
                oracle.blocks[position].id
            ));
        }
    }
    if mismatches.is_empty() {
        Comparison::Agrees {
            blocks: published.len(),
        }
    } else {
        Comparison::Disagrees(mismatches)
    }
}

/// The frames of one body, through the pass under test.
fn published_frames(
    facts: &MethodCodeFacts,
    canonical: &CanonicalCfg,
    method: &FrameMethod<'_>,
) -> FrameOutcome {
    let mut budget = Budget::new(limits());
    frames(facts, canonical, method, &mut budget).expect("the fixture runs inside its budget")
}

/// Generous limits: the fixtures here are tiny and no entry of this module is about the budget.
fn limits() -> jarde_reader::budget::Limits {
    jarde_reader::budget::Limits {
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        elapsed_millis: u64::MAX,
        ..jarde_reader::budget::Limits::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call_context::{CallContextOutcome, call_contexts};
    use crate::canonical::{CanonicalOutcome, canonical_cfg};
    use crate::cfg::raw_cfg;
    use crate::frame::FrameTable;
    use jarde_reader::budget::Budget;
    use jarde_reader::classfile::{class_facts, method_code_facts};
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant, SnapshotId,
    };
    use jarde_reader::view::LoaderId;

    fn budget() -> Budget {
        Budget::new(limits())
    }

    /// One fixture body: the decoded facts, the class file's own constant pool, the canonical
    /// graph, and the declaration facts the frames are derived from.
    struct Body {
        facts: MethodCodeFacts,
        pool: Vec<jarde_reader::classfile::CpEntryFacts>,
        canonical: CanonicalCfg,
        access_flags: u16,
        name: &'static [u8],
        descriptor: &'static [u8],
        owner: &'static [u8],
        super_class: Option<&'static [u8]>,
        loader: LoaderId,
    }

    impl Body {
        fn method(&self) -> FrameMethod<'_> {
            FrameMethod {
                access_flags: self.access_flags,
                name: self.name,
                descriptor: self.descriptor,
                owner: self.owner,
                super_class: self.super_class,
                pool: &self.pool,
                loader: &self.loader,
            }
        }

        fn frames(&self) -> FrameOutcome {
            published_frames(&self.facts, &self.canonical, &self.method())
        }
    }

    /// Every layer up to the frames, over one class file.
    fn body_of(
        bytes: &[u8],
        name: &'static [u8],
        descriptor: &'static [u8],
        access_flags: u16,
    ) -> Body {
        let mut budget = budget();
        let header = class_facts(bytes, &mut budget).expect("the fixture is a class file");
        let member = header
            .methods
            .first()
            .expect("the fixture declares one method");
        let facts = method_code_facts(bytes, member, &mut budget).expect("the fixture decodes");
        let pool = header.constant_pool.clone();
        let raw = raw_cfg(&facts, &mut budget).expect("the fixture has a raw graph");
        let contexts = match call_contexts(&facts, &raw, header.major_version, &mut budget)
            .expect("the call-context walk runs")
        {
            CallContextOutcome::Established(contexts) => contexts,
            other => panic!("the fixture must establish contexts, got {other:?}"),
        };
        let method_id = PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: SnapshotId("test".to_string()),
                },
                class_bytes: ClassBytesId {
                    digest: Digest("test".to_string()),
                    length: 0,
                },
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        };
        let canonical = match canonical_cfg(&facts, &raw, &contexts, &method_id, &mut budget)
            .expect("the budget is ample")
        {
            CanonicalOutcome::Canonical(graph) => *graph,
            CanonicalOutcome::Fallback { message } => {
                panic!("the fixture must normalize, but stopped: {message}")
            }
        };
        Body {
            facts,
            pool,
            canonical,
            access_flags,
            name,
            descriptor,
            owner: b"Test",
            super_class: Some(b"java/lang/Object"),
            loader: LoaderId("app".to_string()),
        }
    }

    /// One class file of `Test` with one method whose `Code` is `code`, at class-file version
    /// `major`, with `handlers` as its catch-all exception table.
    #[allow(clippy::too_many_arguments, reason = "one fixture builder per module")]
    fn class_of(
        major: u16,
        flags: u16,
        name: &[u8],
        descriptor: &[u8],
        code: &[u8],
        max_stack: u16,
        max_locals: u16,
        handlers: &[(u16, u16, u16)],
    ) -> Vec<u8> {
        let mut pool = Vec::new();
        utf8(&mut pool, b"Test");
        class(&mut pool, 1);
        utf8(&mut pool, b"java/lang/Object");
        class(&mut pool, 3);
        utf8(&mut pool, name);
        utf8(&mut pool, descriptor);
        utf8(&mut pool, b"Code");

        let mut content = Vec::new();
        u16b(&mut content, max_stack);
        u16b(&mut content, max_locals);
        content.extend_from_slice(
            &u32::try_from(code.len())
                .expect("a fixture body fits u32")
                .to_be_bytes(),
        );
        content.extend_from_slice(code);
        u16b(
            &mut content,
            u16::try_from(handlers.len()).expect("fixture handlers fit u16"),
        );
        for (start, end, handler) in handlers {
            u16b(&mut content, *start);
            u16b(&mut content, *end);
            u16b(&mut content, *handler);
            u16b(&mut content, 0);
        }
        u16b(&mut content, 0);

        let mut method = Vec::new();
        u16b(&mut method, flags);
        u16b(&mut method, 5);
        u16b(&mut method, 6);
        u16b(&mut method, 1);
        u16b(&mut method, 7);
        method.extend_from_slice(
            &u32::try_from(content.len())
                .expect("fixture Code content fits u32")
                .to_be_bytes(),
        );
        method.extend_from_slice(&content);

        let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
        u16b(&mut bytes, 0);
        u16b(&mut bytes, major);
        u16b(&mut bytes, 8);
        bytes.extend_from_slice(&pool);
        u16b(&mut bytes, 0x0021);
        u16b(&mut bytes, 2);
        u16b(&mut bytes, 4);
        u16b(&mut bytes, 0);
        u16b(&mut bytes, 0);
        u16b(&mut bytes, 1);
        bytes.extend_from_slice(&method);
        u16b(&mut bytes, 0);
        bytes
    }

    fn u16b(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn utf8(pool: &mut Vec<u8>, text: &[u8]) {
        pool.push(1);
        u16b(
            pool,
            u16::try_from(text.len()).expect("a fixture name fits u16"),
        );
        pool.extend_from_slice(text);
    }

    fn class(pool: &mut Vec<u8>, name: u16) {
        pool.push(7);
        u16b(pool, name);
    }

    /// The descriptor the exception fixtures use: one reference in, one out, so their handlers
    /// carry a value the frames state.
    const EXCEPTION_METHOD: &[u8] = b"(Ljava/lang/Object;)Ljava/lang/Object;";

    /// A diamond: a condition, two arms, one join, and a loop back to the head.
    fn diamond_and_loop() -> Body {
        let asm = vec![
            0x04, 0x3b, // 0: iconst_1; 1: istore_0
            0x1a, 0x06, 0xa2, 0x00, 0x09, // 2: iload_0; 3: iconst_3; 4: if_icmpge 13
            0x84, 0x00, 0x01, // 7: iinc 0, 1
            0xa7, 0xff, 0xf8, // 10: goto 2
            0x1a, 0xac, // 13: iload_0; 14: ireturn
        ];
        let class = class_of(49, 0x0009, b"method", b"()I", &asm, 2, 1, &[]);
        body_of(&class, b"method", b"()I", 0x0009)
    }

    /// The wide forms and both switch forms: `wide istore/iload/iinc` on local 300, a
    /// `tableswitch` and a `lookupswitch`.
    fn wide_and_switch() -> Body {
        let mut code = Vec::new();
        // Fix-ups are `(operand position, BCI of the switch opcode, target position)`: a switch
        // operand is relative to the opcode, not to the operand.
        let mut fixups: Vec<(usize, usize, usize)> = Vec::new();
        code.push(0x04); // 0: iconst_1
        code.extend_from_slice(&[0xc4, 0x36, 0x01, 0x2c]); // 1: wide istore 300
        code.extend_from_slice(&[0xc4, 0x84, 0x01, 0x2c, 0xff, 0xfd]); // 5: wide iinc 300, -3
        code.extend_from_slice(&[0xc4, 0x15, 0x01, 0x2c]); // 11: wide iload 300
        let switch_at = code.len();
        code.push(0xaa); // 15: tableswitch over {-2, -1}
        while !code.len().is_multiple_of(4) {
            code.push(0);
        }
        let default_operand = code.len();
        code.extend_from_slice(&[0; 4]);
        code.extend_from_slice(&(-2_i32).to_be_bytes());
        code.extend_from_slice(&(-1_i32).to_be_bytes());
        let case0 = code.len();
        code.extend_from_slice(&[0; 4]);
        let case1 = code.len();
        code.extend_from_slice(&[0; 4]);
        let t0 = code.len();
        code.push(0xb1); // return
        let t1 = code.len();
        code.push(0x04); // iconst_1
        let inner_at = code.len();
        code.push(0xab); // lookupswitch over {1}
        while !code.len().is_multiple_of(4) {
            code.push(0);
        }
        let inner_default = code.len();
        code.extend_from_slice(&[0; 4]);
        code.extend_from_slice(&1_i32.to_be_bytes()); // npairs
        code.extend_from_slice(&1_i32.to_be_bytes()); // the only key
        let inner_case = code.len();
        code.extend_from_slice(&[0; 4]);
        let a = code.len();
        code.push(0xb1); // return
        let default_block = code.len();
        code.push(0xb1); // return
        fixups.push((default_operand, switch_at, default_block));
        fixups.push((case0, switch_at, t0));
        fixups.push((case1, switch_at, t1));
        fixups.push((inner_default, inner_at, default_block));
        fixups.push((inner_case, inner_at, a));
        for (operand, opcode, target) in fixups {
            let offset = i32::try_from(target).expect("a fixture body fits i32")
                - i32::try_from(opcode).expect("a fixture body fits i32");
            code[operand..operand + 4].copy_from_slice(&offset.to_be_bytes());
        }
        let class = class_of(52, 0x0009, b"method", b"()V", &code, 1, 301, &[]);
        body_of(&class, b"method", b"()V", 0x0009)
    }

    /// A body over a category-2 parameter: the long takes two local slots, is moved between them
    /// and discarded. The oracle models the pair itself, so this is the fixture for the category
    /// the rest of the covered bodies never touch.
    fn category_two() -> Body {
        const CODE: &[u8] = &[
            0x1e, // 0: lload_0
            0x41, // 1: lstore_2
            0x20, // 2: lload_2
            0x58, // 3: pop2
            0xb1, // 4: return
        ];
        let class = class_of(49, 0x0009, b"method", b"(J)V", CODE, 2, 4, &[]);
        body_of(&class, b"method", b"(J)V", 0x0009)
    }

    /// A record whose range starts **inside** a block, and one that starts on a block, each
    /// covering one throwing instruction.
    fn mid_block_range() -> Body {
        const CODE: &[u8] = &[
            0x04, 0x03, 0x6c, 0x57, // 0: iconst_1; iconst_0; idiv; pop
            0xa7, 0x00, 0x03, // 4: goto 7
            0x04, 0x03, 0x6c, 0x57, // 7: iconst_1; iconst_0; idiv; pop
            0xb1, // 11: return
            0x57, 0xb1, // 12: pop; return
            0x57, 0xb1, // 14: pop; return
        ];
        let class = class_of(
            49,
            0x0009,
            b"method",
            b"()V",
            CODE,
            2,
            1,
            &[(1, 4, 12), (7, 10, 14)],
        );
        body_of(&class, b"method", b"()V", 0x0009)
    }

    /// The exception edge that runs back into a block that was already processed, with the throw
    /// site's locals differing from the block's exit on the first visit: R9's own body.
    fn exception_back_edge() -> Body {
        const CODE: &[u8] = &[
            0x01, 0x4c, // 0: aconst_null; 1: astore_1
            0x04, 0x03, 0x6c, 0x57, // 2: iconst_1; iconst_0; idiv; pop
            0x2a, 0x4c, 0x2a, 0xc7, 0xff,
            0xf9, // 6: aload_0; 7: astore_1; 8: aload_0; 9: ifnonnull 2
            0x2b, 0xb0, // 12: aload_1; 13: areturn
            0x57, 0xa7, 0xff, 0xf3, // 14: pop; 15: goto 2
        ];
        let class = class_of(
            49,
            0x0009,
            b"method",
            EXCEPTION_METHOD,
            CODE,
            2,
            2,
            &[(2, 12, 14)],
        );
        body_of(&class, b"method", EXCEPTION_METHOD, 0x0009)
    }

    /// Two throw sites of one block under one record: one raw edge, two logical inputs.
    fn two_throw_sites_one_block() -> Body {
        const CODE: &[u8] = &[
            0x01, 0x4c, // 0: aconst_null; 1: astore_1
            0x04, 0x03, 0x6c, 0x57, // 2: iconst_1; iconst_0; idiv; pop
            0x2a, 0x4c, 0x2a, 0x4d, // 6: aload_0; 7: astore_1; 8: aload_0; 9: astore_2
            0x04, 0x03, 0x6c, 0x57, // 10: iconst_1; iconst_0; idiv; pop
            0x2a, 0xc7, 0xff, 0xf3, // 14: aload_0; 15: ifnonnull 2
            0x2b, 0xb0, // 18: aload_1; 19: areturn
            0x57, 0x2b, 0xb0, // 20: pop; 21: aload_1; 22: areturn
        ];
        let class = class_of(
            49,
            0x0009,
            b"method",
            EXCEPTION_METHOD,
            CODE,
            2,
            3,
            &[(2, 18, 20)],
        );
        body_of(&class, b"method", EXCEPTION_METHOD, 0x0009)
    }

    /// A `jsr`/`ret` body: the family outside the oracle's subset.
    fn legacy_clone() -> Body {
        const CODE: &[u8] = &[
            0xa8, 0x00, 0x07, // 0: jsr 7
            0xa8, 0x00, 0x04, // 3: jsr 7
            0xb1, // 6: return
            0x4b, // 7: astore_0
            0xa9, 0x00, // 8: ret 0
        ];
        let class = class_of(50, 0x0009, b"method", b"()V", CODE, 1, 1, &[]);
        body_of(&class, b"method", b"()V", 0x0009)
    }

    /// The comparison of one body, with the published table it produced.
    fn comparison(body: &Body, flaws: &Flaws) -> Comparison {
        match body.frames() {
            FrameOutcome::Frames(table) => compare(
                &body.facts,
                &body.canonical,
                &body.method(),
                table.blocks(),
                flaws,
            ),
            other => panic!("the fixture must be analyzed, got {other:?}"),
        }
    }

    /// The published table of one body, for the tests that tamper with it.
    fn published_table(body: &Body) -> Box<FrameTable> {
        match body.frames() {
            FrameOutcome::Frames(table) => table,
            other => panic!("the fixture must be analyzed, got {other:?}"),
        }
    }

    fn agrees(body: &Body) -> usize {
        match comparison(body, &Flaws::default()) {
            Comparison::Agrees { blocks } => blocks,
            other => panic!("the oracle must agree with the published table, got {other:?}"),
        }
    }

    /// The disagreements one flaw produces, or a panic that says it produced none.
    fn disagrees(body: &Body, flaws: &Flaws) -> Vec<String> {
        match comparison(body, flaws) {
            Comparison::Disagrees(mismatches) => mismatches,
            other => panic!("the flaw must turn the comparison red, got {other:?}"),
        }
    }

    #[test]
    fn the_plain_bodies_agree_with_the_published_table() {
        let bodies = [
            ("diamond and loop", diamond_and_loop()),
            ("wide and switch", wide_and_switch()),
            ("category two", category_two()),
        ];
        for (name, body) in &bodies {
            let blocks = agrees(body);
            assert!(
                blocks >= 1,
                "{name}: the comparison covered {blocks} blocks"
            );
        }
    }

    #[test]
    fn a_range_starting_inside_a_block_agrees_on_the_handler_entry() {
        // The shape 4.2b's third fix closed: the record's range begins in the middle of a block,
        // so the handler's entry state comes from the throw site the range covers rather than
        // from the block's start. The oracle reaches the handler through the published edge.
        let body = mid_block_range();
        let blocks = agrees(&body);
        assert!(
            blocks >= 3,
            "the body has the two blocks, their handlers and the join: {blocks} entries"
        );
    }

    #[test]
    fn an_exception_back_edge_agrees_on_the_state_it_settles_on() {
        // R9's body: the throw site's locals move while the block's exit does not, so the
        // oracle's states can only come from the input that really changed.
        let body = exception_back_edge();
        let blocks = agrees(&body);
        assert!(blocks >= 3, "{blocks} entries");
    }

    #[test]
    fn two_throw_sites_of_one_block_agree_with_the_merge_of_both() {
        // One raw edge, two logical inputs: the handler's entry state is the merge of the states
        // the two sites hand it, and the oracle computes both of them itself.
        let body = two_throw_sites_one_block();
        let blocks = agrees(&body);
        assert!(blocks >= 3, "{blocks} entries");
    }

    #[test]
    fn the_oracles_own_raise_classification_finds_the_graphs_throw_sites() {
        // The oracle classifies which instructions may raise on its own, and only uses the
        // graph's throw sites to hand a handler its inputs. The two must agree about *which*
        // instructions those are, otherwise the oracle would be taking the wrong snapshots.
        for body in [
            mid_block_range(),
            exception_back_edge(),
            two_throw_sites_one_block(),
        ] {
            let oracle = Oracle::build(&body.facts, &body.canonical, &body.method())
                .expect("the body is inside the subset");
            for block in &oracle.blocks {
                let mut stated: Vec<u32> = body
                    .canonical
                    .throw_sites
                    .iter()
                    .filter(|site| site.block == block.id)
                    .map(|site| site.bci)
                    .collect();
                stated.sort_unstable();
                stated.dedup();
                assert_eq!(
                    block.sites, stated,
                    "{:?}: the oracle's raisers and the graph's throw sites agree",
                    block.id
                );
            }
        }
    }

    #[test]
    fn the_legacy_clone_body_abstains_with_its_reason() {
        let body = legacy_clone();
        match comparison(&body, &Flaws::default()) {
            Comparison::Abstained(Abstain::LegacyCalls) => {}
            other => panic!("a `jsr` body is outside the subset, got {other:?}"),
        }
    }

    #[test]
    fn a_body_outside_the_subset_abstains_with_the_instruction_that_is_outside_it() {
        // `new` is outside the subset, and the abstention names the instruction instead of
        // silently comparing a body the oracle cannot read.
        let mut code = Vec::new();
        code.extend_from_slice(&[0xbb, 0x00, 0x02]); // 0: new #2
        code.extend_from_slice(&[0x59]); // 3: dup
        code.extend_from_slice(&[0xb7, 0x00, 0x14]); // 4: invokespecial #20
        code.extend_from_slice(&[0xb1]); // 7: return
        let mut pool = Vec::new();
        utf8(&mut pool, b"Test");
        class(&mut pool, 1);
        utf8(&mut pool, b"java/lang/Object");
        class(&mut pool, 3);
        utf8(&mut pool, b"method");
        utf8(&mut pool, b"()V");
        utf8(&mut pool, b"Code");
        utf8(&mut pool, b"<init>");
        utf8(&mut pool, b"()V");
        pool.push(12);
        u16b(&mut pool, 8);
        u16b(&mut pool, 9);
        pool.push(10);
        u16b(&mut pool, 2);
        u16b(&mut pool, 10);
        let class = class_of(49, 0x0009, b"method", b"()V", &code, 2, 0, &[]);
        // The class file above is the one `class_of` builds; the pool entries it holds are not
        // the ones this body names, so the body is refused by the reader first. The assertion is
        // about the oracle's boundary, so it is made on the instruction classification directly.
        assert!(!covered(0xbb), "`new` is outside the subset");
        assert_eq!(
            compare_instruction(0xbb),
            Some(Abstain::Instruction {
                bci: 0,
                opcode: 0xbb
            })
        );
        let _ = (class, pool);
    }

    /// The abstention one instruction produces, through the same classification the oracle uses.
    fn compare_instruction(opcode: u8) -> Option<Abstain> {
        if covered(opcode) {
            None
        } else {
            Some(Abstain::Instruction { bci: 0, opcode })
        }
    }

    #[test]
    fn the_oracle_never_names_a_transfer_helper_of_the_frame_pass() {
        // The scan keeps the independence honest: the oracle may read the frame pass's types and
        // its published table, but not a transfer, merge or replay helper of it.
        let source = include_str!("frame_oracle.rs");
        for token in [
            concat!("apply", "_form"),
            concat!("merge", "_frame"),
            concat!("merge", "_local"),
            concat!("block", "_touches"),
            concat!("entry", "_slots"),
            concat!("starts", "_value"),
            concat!("replay", "("),
            concat!("merge", "_stack"),
            concat!("instruction", "_indices"),
            concat!("transfer", "_block"),
        ] {
            assert!(
                !source.contains(token),
                "the oracle names the production helper {token:?}"
            );
        }
    }

    #[test]
    fn flaw_one_reading_the_aggregated_edge_is_caught_by_the_two_site_body() {
        // The flaw hands the handler the source block's **exit** where the state its throw site
        // was in belongs. One raw edge can cover several sites whose states differ, so the body
        // that catches it is the one with two sites in one block: the aggregate reading reaches
        // the handler with a second definition of the slot the published table leaves `Top`.
        //
        // R9's own back-edge body deliberately does **not** catch this flaw, and the reason is
        // worth stating: at that body's fixed point the site's state equals the block's exit
        // (the overwrite happens before the exit either way), so the difference R9 found was a
        // *transient* of the visiting order — which is what the one-pass flaw below models.
        let flaws = Flaws {
            aggregate_exception_inputs: true,
            ..Flaws::default()
        };
        let two_sites = two_throw_sites_one_block();
        let red = disagrees(&two_sites, &flaws);
        assert!(
            red.iter().any(|message| message.contains("local slot 2")),
            "the two-site body catches the aggregated reading: {red:?}"
        );
        let back_edge = exception_back_edge();
        assert_eq!(
            comparison(&back_edge, &flaws),
            comparison(&back_edge, &Flaws::default()),
            "at the back-edge body's fixed point both readings agree, so this body is not the one \
             that catches this flaw"
        );
    }

    #[test]
    fn flaw_two_a_single_pass_over_the_blocks_is_caught_by_the_back_edge_body() {
        // A fixed point that stops after the first visit of each block is exactly the shape R9's
        // defect had: the block that hands a handler its state was never re-processed once its
        // own entry changed, so the handler kept the stale input.
        let flaws = Flaws {
            one_pass: true,
            ..Flaws::default()
        };
        let back_edge = exception_back_edge();
        let red = disagrees(&back_edge, &flaws);
        assert!(
            red.iter().any(|message| message.contains("local slot 1")),
            "the back-edge body catches a single visit per block: {red:?}"
        );
        // The plain loop body does **not** catch it: its loop header settles on the same value
        // whichever order the blocks are visited in, so a single visit is enough there. The
        // exception back edge is the shape where the order matters.
        assert_eq!(
            comparison(&diamond_and_loop(), &flaws),
            comparison(&diamond_and_loop(), &Flaws::default()),
            "the plain loop body does not distinguish the visiting orders"
        );
    }

    #[test]
    fn flaw_three_a_comparator_that_reads_one_slot_less_is_caught() {
        // The comparator that skips one local slot misses exactly the differences that live in
        // it: the slot R9's body differs in is planted with a value the body does not imply, and
        // only the honest comparator finds it.
        let body = exception_back_edge();
        let table = published_table(&body);
        let mut tampered: Vec<BlockFrame> = table.blocks().to_vec();
        let mut planted = None;
        for frame in &mut tampered {
            let slot = frame
                .locals
                .iter_mut()
                .position(|value| !matches!(value, Value::Top | Value::Second));
            if let Some(slot) = slot {
                frame.locals[slot] = Value::Top;
                planted = Some((frame.block.clone(), slot));
                break;
            }
        }
        let (block, slot) = planted.expect("the fixture states at least one local value");
        let honest = compare(
            &body.facts,
            &body.canonical,
            &body.method(),
            &tampered,
            &Flaws::default(),
        );
        let skipping = Flaws {
            drop_a_local: Some(slot),
            ..Flaws::default()
        };
        let weakened = compare(
            &body.facts,
            &body.canonical,
            &body.method(),
            &tampered,
            &skipping,
        );
        assert!(
            matches!(honest, Comparison::Disagrees(_)),
            "the comparison reads {block:?} slot {slot} and finds the planted value: {honest:?}"
        );
        assert_eq!(
            weakened,
            Comparison::Agrees {
                blocks: tampered.len()
            },
            "a comparator that skips {block:?} slot {slot} cannot see what was planted in it"
        );
    }

    #[test]
    fn flaw_four_trusting_the_published_table_is_caught_by_a_tampered_table() {
        // The weakening that would make this module a restatement of the pass: answer "agrees"
        // without looking. The tampered table is what catches it — the comparison finds a value
        // the body's own instructions do not imply, and the weakened oracle does not.
        let body = exception_back_edge();
        let table = published_table(&body);
        let mut tampered: Vec<BlockFrame> = table.blocks().to_vec();
        let mut changed = false;
        for frame in &mut tampered {
            if let Some(slot) = frame.locals.get_mut(1)
                && !matches!(slot, Value::Top)
            {
                *slot = Value::Top;
                changed = true;
                break;
            }
        }
        assert!(
            changed,
            "the fixture states a value in slot 1 to tamper with"
        );
        let flawed = Flaws {
            trust_the_published_table: true,
            ..Flaws::default()
        };
        let honest = compare(
            &body.facts,
            &body.canonical,
            &body.method(),
            &tampered,
            &Flaws::default(),
        );
        let weakened = compare(
            &body.facts,
            &body.canonical,
            &body.method(),
            &tampered,
            &flawed,
        );
        assert!(
            matches!(honest, Comparison::Disagrees(_)),
            "the comparison finds the planted value: {honest:?}"
        );
        assert_eq!(
            weakened,
            Comparison::Agrees {
                blocks: tampered.len()
            },
            "the weakened comparison trusts whatever the table says, which is what the honest \
             one must not do"
        );
    }

    #[test]
    fn the_model_round_trips_every_value_the_covered_bodies_state() {
        // Non-vacuity of the comparison: the covered bodies really state values in both
        // categories, so "agrees" is not the agreement of two empty statements.
        let mut ints = 0;
        let mut references = 0;
        let mut second = 0;
        for body in [
            diamond_and_loop(),
            wide_and_switch(),
            category_two(),
            mid_block_range(),
            exception_back_edge(),
            two_throw_sites_one_block(),
        ] {
            let table = published_table(&body);
            for frame in table.blocks() {
                for value in frame.locals.iter().chain(frame.stack.iter()) {
                    match project(value) {
                        Some(OValue::Int) => ints += 1,
                        Some(OValue::Ref(_)) => references += 1,
                        Some(OValue::Second) => second += 1,
                        Some(OValue::Null) => references += 1,
                        _ => {}
                    }
                }
            }
        }
        assert!(ints > 0, "the covered bodies state ints");
        assert!(references > 0, "and references");
        assert!(second > 0, "and the upper half of a category-2 pair");
    }

    #[test]
    fn the_category_two_body_agrees_and_states_both_of_its_slots() {
        // A category-2 value is one value in the model and two slots in the state, which is where
        // a naive reading usually goes wrong: the pair is checked on both sides.
        let body = category_two();
        let table = published_table(&body);
        let entry = table.blocks().first().expect("the entry block has a state");
        assert!(
            matches!(project(&entry.locals[0]), Some(OValue::Long)),
            "the long parameter is in slot 0: {:?}",
            entry.locals
        );
        assert_eq!(
            project(&entry.locals[1]),
            Some(OValue::Second),
            "and its upper half in slot 1: {:?}",
            entry.locals
        );
        let blocks = agrees(&body);
        assert!(blocks >= 1, "{blocks} entries");
    }

    #[test]
    fn the_published_table_of_a_covered_body_has_the_blocks_the_oracle_reaches() {
        // The comparison is over block entries, so the two must agree about *which* blocks have
        // one: a table that quietly dropped a block would otherwise compare fewer entries and
        // look green.
        let body = two_throw_sites_one_block();
        let table = published_table(&body);
        let oracle = Oracle::build(&body.facts, &body.canonical, &body.method())
            .expect("the body is inside the subset");
        let entries = oracle
            .fixed_point(&Flaws::default())
            .expect("the oracle's own fixed point converges");
        let reached: Vec<&CanonicalBlockId> = entries
            .iter()
            .enumerate()
            .filter(|(_, state)| state.is_some())
            .map(|(position, _)| &oracle.blocks[position].id)
            .collect();
        assert_eq!(
            reached.len(),
            table.blocks().len(),
            "the two readings reach the same number of blocks"
        );
        for frame in table.blocks() {
            assert!(
                reached.contains(&&frame.block),
                "{:?} is published and the oracle reaches it too",
                frame.block
            );
        }
    }

    #[test]
    fn the_declaration_parser_reads_the_descriptors_the_fixtures_use() {
        assert_eq!(parse_descriptor(b"()V").expect("parsable"), Vec::new());
        assert_eq!(
            parse_descriptor(b"(I)J").expect("parsable"),
            vec![OValue::Int],
            "the return type is not a parameter"
        );
        assert_eq!(
            parse_descriptor(b"(Ljava/lang/Object;[I)Ljava/lang/Object;").expect("parsable"),
            vec![
                OValue::Ref(Some(b"java/lang/Object".to_vec())),
                OValue::Ref(Some(b"[I".to_vec()))
            ]
        );
        assert_eq!(
            parse_descriptor(b"([[J[DZD)Ljava/lang/Object;").expect("parsable"),
            vec![
                OValue::Ref(Some(b"[[J".to_vec())),
                OValue::Ref(Some(b"[D".to_vec())),
                OValue::Int,
                OValue::Double,
            ]
        );
    }

    #[test]
    fn the_entry_state_follows_the_declaration() {
        let body = two_throw_sites_one_block();
        let oracle = Oracle::build(&body.facts, &body.canonical, &body.method())
            .expect("the body is inside the subset");
        let state = oracle.entry_state().expect("a static method is modelled");
        assert_eq!(state.locals.len(), 3, "max_locals of the fixture");
        assert_eq!(
            state.locals[0],
            OValue::Ref(Some(b"java/lang/Object".to_vec())),
            "the first parameter is the reference the descriptor names"
        );
    }
}
