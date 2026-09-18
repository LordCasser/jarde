//! Descriptor-driven frames: the slot state every canonical block is entered with (4.1).
//!
//! This is the pass of `IrPhase::Frame`. For every block of the canonical graph it answers what
//! the method's frame looks like when that block starts — its local slots and its operand stack,
//! both counted in **slots** — by walking the block's instructions and applying what each opcode
//! does. Three things decide that, and none of them is rendered bytecode text:
//!
//! * the opcode, through the **dense table** below ([`TABLE`]), which names each instruction's
//!   pop and push sequence in slot classes;
//! * the reader's typed operands ([`InstructionOperands`]), which name the local of a load, a
//!   store or `ret`, the `multianewarray` dimension count and the constant-pool index;
//! * the class file's own descriptors for everything the opcode alone cannot decide — an
//!   `invoke*` argument and return shape, a field access, an `ldc` constant, `checkcast` — read
//!   from the constant pool the caller hands in.
//!
//! No `StackMapTable`, `LineNumberTable` or `LocalVariableTable` is read or needed: the state is
//! *derived* from the descriptors and the data flow, which is exactly why a body without debug
//! attributes — or without stack maps at all — still gets frames. Nothing here is a verifier:
//! the report's `verification` plane stays `NotPerformed` and the semantic evidence stays
//! `Unproven`, whatever this pass proves locally.
//!
//! # What one slot holds
//!
//! The lattice ([`Value`]) keeps apart the states a JVM frame really has to tell apart, because
//! collapsing any of them loses the fact a later consumer needs:
//!
//! * [`Value::Top`] is **not readable**: a local no path wrote, or one whose paths disagree. A
//!   read of it is the bytecode's own contradiction, not a gap of this build;
//! * [`Value::Second`] is the upper half of a category-2 value: it is the lower slot's, and
//!   reading it on its own is as wrong as reading an unreadable local — which is why it is a
//!   state of its own and not another `Top`;
//! * [`Value::UninitializedThis`] and [`Value::Uninitialized`] carry the new-site BCI, so two
//!   allocations are different values and never one;
//! * [`Value::Null`] is the null type, and [`Value::Ref`] is an initialized reference whose type
//!   is either **named** — the class file spells it and the loader identity travels with it — or
//!   unknown. An unknown reference is conservative but is *not* `Top` and is never a basic type:
//!   a slot that holds "some object" can never be read as an `int`.
//!
//! # What one block entry is
//!
//! Each block's entry state is the **merge** of its predecessors' exit states, computed by a
//! worklist to a fixpoint. The two halves of a frame merge under different rules, on purpose:
//!
//! * the **operand stack** must agree in depth and in slot classes: a merge of a depth-1 stack
//!   with a depth-2 one, or of an `int` slot with a reference slot, is a contradiction of the
//!   body (`ir_frame_inconsistent`). References merge conservatively: two different named types
//!   merge to an unknown reference, `null` with a named reference is that reference;
//! * the **locals** are allowed to disagree, and an undefined or disagreeing slot becomes
//!   [`Value::Top`] instead of failing: a local that is written on both paths and read on
//!   neither is not a reason to refuse a legal method. The failure is moved to where it belongs,
//!   the read of that slot.
//!
//! Category-2 binding is explicit: a `long`/`double` occupies two slots, and writing **either**
//! half of the pair invalidates the other half, which becomes `Top` — the previous two-slot
//! value no longer exists, and no stale lower half may survive a store into its upper slot.
//!
//! # The 4.1 boundary
//!
//! The initialization conversions — flipping `uninitializedThis`/new-site aliases after a
//! successful `invokespecial <init>`, and the handler entries, whose locals snapshot comes from
//! per-throw-site state — are **4.2's** work, not this slice's. 4.1 therefore states the
//! distinctions and stops where a body needs them:
//!
//! * an uninitialized value may only be **moved** by a pure stack operation (`astore`/`aload`,
//!   the `dup*` family, `pop*`, `swap`); any other consumer of one stops the body;
//! * a block entered through an exception edge gets no invented entry state: the canonical
//!   graph's exception edge aggregates a block's throw sites, and a state fabricated from it
//!   would be exactly the "one state for several BCIs" mistake the design forbids.
//!
//! Both stops are [`FrameOutcome::Unsupported`] — this build does not prove the state yet —
//! and the driver reports them under `ir_frame_deferred`, which is a different fact from
//! the [`FrameOutcome::Inconsistent`] a contradiction of the bytes produces. Silently skipping
//! either case, or reporting it as a contradiction, is what the split exists to prevent.

use std::collections::BTreeMap;
use std::collections::VecDeque;

use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{CpEntryFacts, CpEntryKind, InstructionOperands, MethodCodeFacts};
use jarde_reader::error::{Error, Result};
use jarde_reader::view::LoaderId;

use crate::canonical::{CanonicalBlock, CanonicalBlockId, CanonicalCfg, CanonicalEdgeKind};

/// Stop code of a body this build cannot state the frames of yet.
///
/// It means "4.2's initialization or handler-entry analysis is missing here", never "the
/// bytecode is wrong": the driver publishes the phases before this one and no `Frames` fact.
pub(crate) const IR_FRAME_DEFERRED: &str = "ir_frame_deferred";

/// Stop code of a body whose bytes contradict themselves.
///
/// A stack whose paths disagree in depth or in slot class, a read of a slot that holds no
/// readable value, a constant-pool operand that names the wrong kind of entry: each of those is
/// something the method's own bytes decide, and none of them is a limit of this build.
pub(crate) const IR_FRAME_INCONSISTENT: &str = "ir_frame_inconsistent";

/// Deepest operand stack one frame may have, in slots (JVMS 4.11: a `u2` bound, the same ceiling
/// `max_stack` of the `Code` attribute is declared in).
///
/// The bound is checked on every push: a body whose stack would grow past it is inconsistent, and
/// the check is also what keeps a block's derived state finite while the worklist runs.
const MAX_STACK_SLOTS: usize = 65_535;

/// `ACC_STATIC` of the member's access flags: a static method has no `this` in local 0.
const ACC_STATIC: u16 = 0x0008;

/// The one member fact the entry state needs, plus the loader anchor of its references.
///
/// The frame pass cannot read this out of the decoded body: whether the method is static, how it
/// is called and what class it belongs to are declaration facts. Keeping them in one borrowed
/// view is what makes the pass's own signature about the body, the graph and the method.
pub(crate) struct FrameMethod<'a> {
    /// Access flags of the member, as its declaration states them.
    pub(crate) access_flags: u16,
    /// Raw name of the member: `<init>` is what makes local 0 `uninitializedThis`.
    pub(crate) name: &'a [u8],
    /// Raw descriptor of the member: it decides which slots the parameters occupy.
    pub(crate) descriptor: &'a [u8],
    /// Internal name of the declaring class, the type of an initialized `this`.
    pub(crate) owner: &'a [u8],
    /// The class file's own constant pool, the source of every descriptor the opcode alone
    /// cannot decide: the `invoke*` shapes, the field accesses, `ldc`, `checkcast` and the array
    /// creations. It is the header read's fact, handed over instead of read a second time.
    pub(crate) pool: &'a [CpEntryFacts],
    /// The loader a named reference is anchored to **for this request**.
    ///
    /// It is the request's declared load-domain loader, not a resolution result: which loader
    /// really defines the named type is a question this slice does not answer, and a consumer
    /// that has an answer of its own must not read this anchor as the defining loader.
    pub(crate) loader: &'a LoaderId,
}

/// One slot class the frames distinguish.
///
/// `Int` is the category-1 int-like class (`int`, `boolean`, `byte`, `char`, `short`), `Float`
/// the category-1 floating one, `Long`/`Double` the category-2 classes, `Ref` the reference
/// class (`null` included). The distinction inside a category is kept because a body that feeds
/// an `int` where a `float` belongs is as wrong as one that feeds a reference there.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Ty {
    Int,
    Float,
    Long,
    Double,
    Ref,
}

impl Ty {
    /// Slots this class occupies, which is also the stack depth it changes by.
    fn slots(self) -> usize {
        match self {
            Self::Long | Self::Double => 2,
            Self::Int | Self::Float | Self::Ref => 1,
        }
    }

    /// Whether a value of this class fits the slot class, for the error message of a refusal.
    fn accepts(self, value: &Value) -> bool {
        match self {
            Self::Int => *value == Value::Int,
            Self::Float => *value == Value::Float,
            Self::Long => *value == Value::Long,
            Self::Double => *value == Value::Double,
            Self::Ref => {
                matches!(value, Value::Null | Value::Ref(_) | Value::ReturnAddress)
            }
        }
    }
}

/// A reference's type, as far as these facts establish it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RefType {
    /// The class file spells the type at the instruction or in a descriptor — an array creation,
    /// `checkcast`, `this`, a reference parameter — together with the loader anchor of the
    /// request; see [`FrameMethod::loader`].
    Named {
        name: Vec<u8>,
        loader: Box<LoaderId>,
    },
    /// The slot holds a reference whose type these facts do not establish: a `ldc` constant that
    /// names no class at the instruction, or the merge of two differently named references.
    /// Conservative on purpose, and never confused with an unreadable slot or a basic type.
    Unknown,
}

/// One slot's state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Value {
    /// No readable value: an unwritten or disagreeing local, or a slot a merge gave up on.
    Top,
    /// The upper slot of the category-2 value held in the slot below.
    Second,
    Int,
    Float,
    Long,
    Double,
    /// The null type: a reference type that is not a class, and merges with references.
    Null,
    /// An initialized reference; see [`RefType`].
    Ref(RefType),
    /// The uninitialized `this` of a constructor, before its own constructor call.
    UninitializedThis,
    /// An uninitialized value whose `new` runs at this BCI.
    Uninitialized {
        new_site: u32,
    },
    /// The return address a `jsr` pushes, which `astore`/`aload` may carry like a reference.
    ReturnAddress,
}

impl Value {
    /// Slots this value occupies where it is the first slot of the value.
    fn slots(&self) -> usize {
        match self {
            Self::Long | Self::Double => 2,
            _ => 1,
        }
    }

    /// Whether this value is one of the uninitialized states, which 4.1 may only move.
    fn is_uninitialized(&self) -> bool {
        matches!(self, Self::UninitializedThis | Self::Uninitialized { .. })
    }
}

/// What the dense table says an instruction pushes, before the value exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Produced {
    Int,
    Float,
    Long,
    Double,
    /// A reference the instruction does not name (a `ldc` constant): conservative and unknown.
    Ref,
    /// The null type.
    Null,
    /// The return address of `jsr`/`jsr_w`.
    ReturnAddress,
}

impl Produced {
    /// Slots this production takes.
    #[cfg(test)]
    fn slots(self) -> usize {
        match self {
            Self::Long | Self::Double => 2,
            _ => 1,
        }
    }

    /// The value it produces, named where the class file names one.
    fn value(self, named: Option<RefType>) -> Value {
        match self {
            Self::Int => Value::Int,
            Self::Float => Value::Float,
            Self::Long => Value::Long,
            Self::Double => Value::Double,
            Self::Ref => Value::Ref(named.unwrap_or(RefType::Unknown)),
            Self::Null => Value::Null,
            Self::ReturnAddress => Value::ReturnAddress,
        }
    }
}

/// The `dup`/`pop`/`swap` family, whose shape depends on the values already on the stack.
///
/// Each form is implemented explicitly, because the JVMS defines every one of them by the
/// category pattern it accepts — `dup2` duplicates either two category-1 values or one
/// category-2 one — and a table entry that only said "two slots" would accept the patterns the
/// instruction may not see.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Form {
    /// `pop`: one category-1 value.
    Pop,
    /// `pop2`: one category-2 value, or two category-1 ones.
    Pop2,
    /// `dup`: one category-1 value, duplicated.
    Dup,
    /// `dup_x1`: two category-1 values.
    DupX1,
    /// `dup_x2`: a category-1 value over two category-1 ones, or over one category-2 one.
    DupX2,
    /// `dup2`: two category-1 values, or one category-2 one.
    Dup2,
    /// `dup2_x1`: two category-1 values over one category-1 value, or one category-2 value over
    /// one category-1 one.
    Dup2X1,
    /// `dup2_x2`: two category-1 values over two category-1 ones, over a category-1 value below
    /// a category-2 one, a category-2 value below two category-1 ones, or one category-2 value
    /// below another.
    Dup2X2,
    /// `swap`: two category-1 values, exchanged.
    Swap,
}

/// What an instruction does to the operand stack.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stack {
    /// Not an instruction: the `wide` prefix, an unused opcode, or a reserved one.
    NotAnOpcode,
    /// A fixed pop/push sequence: the opcode decides it whatever the class file says. Both
    /// sequences are written top-first for the pops and bottom-first for the pushes.
    Fixed {
        pops: &'static [Ty],
        pushes: &'static [Produced],
    },
    /// A shape the class file decides; see [`PoolEffect`].
    Constant(PoolEffect),
    /// A shape the operand stack itself decides; see [`Form`].
    Form(Form),
}

/// What an instruction does to one local slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Local {
    /// Nothing: the instruction touches no local.
    None,
    /// Reads the local the operands name, which must hold a value of this class, and pushes it.
    /// The pushed value is the local's own, so an uninitialized value or a return address
    /// survives the move.
    Load(Ty),
    /// Pops a value of this class and writes it into the local the operands name.
    Store(Ty),
    /// Reads and writes the local as an `int` (`iinc`); no operand-stack effect.
    Increment,
    /// Reads the local, which must hold the return address `ret` returns to.
    Return,
}

/// The stack shape a class-file fact decides.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PoolEffect {
    /// `ldc`/`ldc_w`: push what the named constant is.
    Ldc,
    /// `ldc2_w`: push the category-2 constant the entry names.
    Ldc2,
    /// A field access: `target` says an object reference is consumed, `write` says the field
    /// value is consumed instead of produced.
    Field { write: bool, target: bool },
    /// An invocation: `receiver` says an object reference is consumed in front of the arguments.
    /// The descriptor decides the argument slots and the return value, and an `<init>` call
    /// produces nothing here — the initialization conversion of its receiver is 4.2's.
    Invoke { receiver: bool },
    /// `new`: push an uninitialized value whose new-site is this instruction's BCI.
    New,
    /// `newarray`: pop the length, push the array type of the element code.
    NewArray,
    /// `anewarray`: pop the length, push the array type of the named class.
    ANewArray,
    /// `multianewarray`: pop one length per dimension, push the named array type.
    MultiANewArray,
    /// `athrow`: pop the throwable and leave no stack behind.
    AThrow,
    /// `checkcast`: the checked class is the type of the reference that comes out.
    CheckCast,
}

/// One row of the dense opcode table.
///
/// A local access names its stack effect through [`Local`] instead of [`Stack::Fixed`], because
/// the value it moves comes from — or goes into — the slot the operands name: the slot class the
/// access requires is the `Ty` of [`Local::Load`] or [`Local::Store`], and the entry's `stack`
/// field stays empty for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Entry {
    /// What the instruction does to the operand stack.
    stack: Stack,
    /// What it does to a local slot.
    local: Local,
    /// Whether this instruction ends the basic block it belongs to.
    ///
    /// The frame layer needs it for a reason of its own: a canonical node is a **fused chain** of
    /// original blocks, and the instructions of a node are therefore not its BCI span — a chain
    /// that jumps over the arm of a diamond would take that arm's instructions with it. The chain
    /// is walked one original block at a time, each stopping at the instruction that ends it, and
    /// this flag is what says where that is. The unit tests check this flag — and nothing else of
    /// the row — against the raw pass's own block-ender classification.
    ends_block: bool,
}

impl Entry {
    /// A row no opcode of a decodable body can reach.
    const NOT_AN_OPCODE: Entry = Entry {
        stack: Stack::NotAnOpcode,
        local: Local::None,
        ends_block: false,
    };

    /// A row whose stack shape the opcode fixes.
    const fn plain(pops: &'static [Ty], pushes: &'static [Produced]) -> Entry {
        Entry {
            stack: Stack::Fixed { pops, pushes },
            local: Local::None,
            ends_block: false,
        }
    }

    /// A load: the local's value is what enters the stack.
    const fn load(kind: Ty) -> Entry {
        Entry {
            stack: Stack::Fixed {
                pops: &[],
                pushes: &[],
            },
            local: Local::Load(kind),
            ends_block: false,
        }
    }

    /// A store: the value that leaves the stack is what enters the local.
    const fn store(kind: Ty) -> Entry {
        Entry {
            stack: Stack::Fixed {
                pops: &[],
                pushes: &[],
            },
            local: Local::Store(kind),
            ends_block: false,
        }
    }

    /// A row with a local effect of its own and no operand-stack effect.
    const fn local(local: Local) -> Entry {
        Entry {
            stack: Stack::Fixed {
                pops: &[],
                pushes: &[],
            },
            local,
            ends_block: false,
        }
    }

    /// A row whose stack shape the values on the stack decide.
    const fn form(form: Form) -> Entry {
        Entry {
            stack: Stack::Form(form),
            local: Local::None,
            ends_block: false,
        }
    }

    /// A row whose stack shape a class-file fact decides.
    const fn constant(effect: PoolEffect) -> Entry {
        Entry {
            stack: Stack::Constant(effect),
            local: Local::None,
            ends_block: false,
        }
    }

    /// The same row for an instruction that ends the block it belongs to.
    const fn ends(self) -> Entry {
        Entry {
            ends_block: true,
            ..self
        }
    }

    /// The depth change this row has, in slots, when the opcode alone decides it.
    ///
    /// `None` is the same set of rows [`crate::cfg::fixed_stack_delta`] is `None` for: the ones
    /// that need the constant pool. The unit tests of this module compare the two, and the
    /// comparison stops at the **depth**: it counts slots, so a row that pops two `int`s and a row
    /// that pops two references have one and the same delta, and the raw pass's effect facts
    /// cannot tell those rows apart. Which slot **classes** a row pops is therefore held by this
    /// table's own per-opcode cases and not by that cross-check, which also says nothing about the
    /// rows whose delta the pool decides — `multianewarray` among them, its dimension count.
    #[cfg(test)]
    fn fixed_delta(self) -> Option<i32> {
        let (pops, pushes) = match self.stack {
            Stack::NotAnOpcode | Stack::Constant(_) => return None,
            Stack::Form(form) => {
                let slots = form.slot_delta();
                return Some(slots);
            }
            Stack::Fixed { pops, pushes } => (pops, pushes),
        };
        let popped: i32 = pops.iter().copied().map(|ty| ty.slots() as i32).sum();
        let pushed: i32 = pushes
            .iter()
            .copied()
            .map(|entry| entry.slots() as i32)
            .sum();
        match self.local {
            // A load pushes what it reads and a store pops what it writes, so their slot shape
            // is the local's own; `iinc` and `ret` change no stack slot.
            Local::Load(kind) => Some(pushed + kind.slots() as i32 - popped),
            Local::Store(kind) => Some(pushed - popped - kind.slots() as i32),
            Local::None | Local::Increment | Local::Return => Some(pushed - popped),
        }
    }
}

impl Form {
    /// Slots the form changes the stack depth by: constant for every form, which is why the raw
    /// pass can publish one delta for the opcode.
    #[cfg(test)]
    fn slot_delta(self) -> i32 {
        match self {
            Self::Pop => -1,
            Self::Pop2 => -2,
            Self::Dup => 1,
            Self::DupX1 => 1,
            Self::DupX2 => 1,
            Self::Dup2 => 2,
            Self::Dup2X1 => 2,
            Self::Dup2X2 => 2,
            Self::Swap => 0,
        }
    }
}

/// Shorthands for the pop sequences, named after the JVMS stack shapes they describe.
const POP_NONE: &[Ty] = &[];
const POP_I: &[Ty] = &[Ty::Int];
const POP_F: &[Ty] = &[Ty::Float];
const POP_L: &[Ty] = &[Ty::Long];
const POP_D: &[Ty] = &[Ty::Double];
const POP_R: &[Ty] = &[Ty::Ref];
const POP_II: &[Ty] = &[Ty::Int, Ty::Int];
const POP_RR: &[Ty] = &[Ty::Ref, Ty::Ref];
const POP_FF: &[Ty] = &[Ty::Float, Ty::Float];
const POP_LL: &[Ty] = &[Ty::Long, Ty::Long];
const POP_DD: &[Ty] = &[Ty::Double, Ty::Double];
/// A long shift: the `int` shift distance on top, the `long` value below it. JVMS 6.5 writes the
/// two values of `lshl` in the other order — that is the *prose* order of the sentence, not the
/// stack: the distance is the top operand, so this sequence is `[Int, Long]` and not `[Long,
/// Int]`.
const POP_IL: &[Ty] = &[Ty::Int, Ty::Long];
/// An array load: the index on top, the array reference below it.
const POP_IR: &[Ty] = &[Ty::Int, Ty::Ref];
/// An int array store: value, index, array reference.
const POP_IIR: &[Ty] = &[Ty::Int, Ty::Int, Ty::Ref];
const POP_FIR: &[Ty] = &[Ty::Float, Ty::Int, Ty::Ref];
const POP_DIR: &[Ty] = &[Ty::Double, Ty::Int, Ty::Ref];
const POP_LIR: &[Ty] = &[Ty::Long, Ty::Int, Ty::Ref];
const POP_RIR: &[Ty] = &[Ty::Ref, Ty::Int, Ty::Ref];

/// Shorthands for the push sequences.
const PUSH_NONE: &[Produced] = &[];
const PUSH_I: &[Produced] = &[Produced::Int];
const PUSH_F: &[Produced] = &[Produced::Float];
const PUSH_L: &[Produced] = &[Produced::Long];
const PUSH_D: &[Produced] = &[Produced::Double];
const PUSH_R: &[Produced] = &[Produced::Ref];
const PUSH_NULL: &[Produced] = &[Produced::Null];
const PUSH_RET: &[Produced] = &[Produced::ReturnAddress];

/// The dense opcode table: one row per opcode value, `0x00` through `0xff`.
///
/// Dense on purpose: an opcode the frame layer has no row for is a row like any other
/// ([`Entry::NOT_AN_OPCODE`]), so "no opcode is missing" is a property of the table's length and
/// not of a reviewer's memory. The rows follow JVMS 6.5, and the opcode a `wide` prefix wraps is
/// what reaches them ([`InstructionOperands::effective_opcode`]): `0xc4` itself is not an
/// instruction and has no row.
static TABLE: [Entry; 256] = build_table();

/// Builds [`TABLE`] row by row, so the table is one match over the opcode values.
const fn build_table() -> [Entry; 256] {
    let mut table = [Entry::NOT_AN_OPCODE; 256];
    let mut opcode = 0usize;
    while opcode < table.len() {
        table[opcode] = row(opcode as u8);
        opcode += 1;
    }
    table
}

/// The row of one opcode.
const fn row(opcode: u8) -> Entry {
    match opcode {
        0x00 => Entry::plain(POP_NONE, PUSH_NONE),       // nop
        0x01 => Entry::plain(POP_NONE, PUSH_NULL),       // aconst_null
        0x02..=0x08 => Entry::plain(POP_NONE, PUSH_I),   // iconst_m1..iconst_5
        0x09 | 0x0a => Entry::plain(POP_NONE, PUSH_L),   // lconst_0/1
        0x0b..=0x0d => Entry::plain(POP_NONE, PUSH_F),   // fconst_0..2
        0x0e | 0x0f => Entry::plain(POP_NONE, PUSH_D),   // dconst_0/1
        0x10 | 0x11 => Entry::plain(POP_NONE, PUSH_I),   // bipush, sipush
        0x12 | 0x13 => Entry::constant(PoolEffect::Ldc), // ldc, ldc_w
        0x14 => Entry::constant(PoolEffect::Ldc2),       // ldc2_w
        0x15 | 0x1a..=0x1d => Entry::load(Ty::Int),      // iload, iload_0..3
        0x16 | 0x1e..=0x21 => Entry::load(Ty::Long),     // lload, lload_0..3
        0x17 | 0x22..=0x25 => Entry::load(Ty::Float),    // fload, fload_0..3
        0x18 | 0x26..=0x29 => Entry::load(Ty::Double),   // dload, dload_0..3
        0x19 | 0x2a..=0x2d => Entry::load(Ty::Ref),      // aload, aload_0..3
        0x2e => Entry::plain(POP_IR, PUSH_I),            // iaload
        0x2f => Entry::plain(POP_IR, PUSH_L),            // laload
        0x30 => Entry::plain(POP_IR, PUSH_F),            // faload
        0x31 => Entry::plain(POP_IR, PUSH_D),            // daload
        0x32 => Entry::plain(POP_IR, PUSH_R),            // aaload
        0x33..=0x35 => Entry::plain(POP_IR, PUSH_I),     // baload, caload, saload
        0x36 | 0x3b..=0x3e => Entry::store(Ty::Int),     // istore, istore_0..3
        0x37 | 0x3f..=0x42 => Entry::store(Ty::Long),    // lstore, lstore_0..3
        0x38 | 0x43..=0x46 => Entry::store(Ty::Float),   // fstore, fstore_0..3
        0x39 | 0x47..=0x4a => Entry::store(Ty::Double),  // dstore, dstore_0..3
        0x3a | 0x4b..=0x4e => Entry::store(Ty::Ref),     // astore, astore_0..3
        0x4f | 0x54..=0x56 => Entry::plain(POP_IIR, PUSH_NONE), // iastore, bastore, castore, sastore
        0x50 => Entry::plain(POP_LIR, PUSH_NONE),               // lastore
        0x51 => Entry::plain(POP_FIR, PUSH_NONE),               // fastore
        0x52 => Entry::plain(POP_DIR, PUSH_NONE),               // dastore
        0x53 => Entry::plain(POP_RIR, PUSH_NONE),               // aastore
        0x57 => Entry::form(Form::Pop),                         // pop
        0x58 => Entry::form(Form::Pop2),                        // pop2
        0x59 => Entry::form(Form::Dup),                         // dup
        0x5a => Entry::form(Form::DupX1),                       // dup_x1
        0x5b => Entry::form(Form::DupX2),                       // dup_x2
        0x5c => Entry::form(Form::Dup2),                        // dup2
        0x5d => Entry::form(Form::Dup2X1),                      // dup2_x1
        0x5e => Entry::form(Form::Dup2X2),                      // dup2_x2
        0x5f => Entry::form(Form::Swap),                        // swap
        0x60 => Entry::plain(POP_II, PUSH_I),                   // iadd
        0x61 => Entry::plain(POP_LL, PUSH_L),                   // ladd
        0x62 => Entry::plain(POP_FF, PUSH_F),                   // fadd
        0x63 => Entry::plain(POP_DD, PUSH_D),                   // dadd
        0x64 => Entry::plain(POP_II, PUSH_I),                   // isub
        0x65 => Entry::plain(POP_LL, PUSH_L),                   // lsub
        0x66 => Entry::plain(POP_FF, PUSH_F),                   // fsub
        0x67 => Entry::plain(POP_DD, PUSH_D),                   // dsub
        0x68 => Entry::plain(POP_II, PUSH_I),                   // imul
        0x69 => Entry::plain(POP_LL, PUSH_L),                   // lmul
        0x6a => Entry::plain(POP_FF, PUSH_F),                   // fmul
        0x6b => Entry::plain(POP_DD, PUSH_D),                   // dmul
        0x6c => Entry::plain(POP_II, PUSH_I),                   // idiv
        0x6d => Entry::plain(POP_LL, PUSH_L),                   // ldiv
        0x6e => Entry::plain(POP_FF, PUSH_F),                   // fdiv
        0x6f => Entry::plain(POP_DD, PUSH_D),                   // ddiv
        0x70 => Entry::plain(POP_II, PUSH_I),                   // irem
        0x71 => Entry::plain(POP_LL, PUSH_L),                   // lrem
        0x72 => Entry::plain(POP_FF, PUSH_F),                   // frem
        0x73 => Entry::plain(POP_DD, PUSH_D),                   // drem
        0x74 => Entry::plain(POP_I, PUSH_I),                    // ineg
        0x75 => Entry::plain(POP_L, PUSH_L),                    // lneg
        0x76 => Entry::plain(POP_F, PUSH_F),                    // fneg
        0x77 => Entry::plain(POP_D, PUSH_D),                    // dneg
        0x78 => Entry::plain(POP_II, PUSH_I),                   // ishl
        0x79 => Entry::plain(POP_IL, PUSH_L),                   // lshl
        0x7a => Entry::plain(POP_II, PUSH_I),                   // ishr
        0x7b => Entry::plain(POP_IL, PUSH_L),                   // lshr
        0x7c => Entry::plain(POP_II, PUSH_I),                   // iushr
        0x7d => Entry::plain(POP_IL, PUSH_L),                   // lushr
        0x7e => Entry::plain(POP_II, PUSH_I),                   // iand
        0x7f => Entry::plain(POP_LL, PUSH_L),                   // land
        0x80 => Entry::plain(POP_II, PUSH_I),                   // ior
        0x81 => Entry::plain(POP_LL, PUSH_L),                   // lor
        0x82 => Entry::plain(POP_II, PUSH_I),                   // ixor
        0x83 => Entry::plain(POP_LL, PUSH_L),                   // lxor
        0x84 => Entry::local(Local::Increment),                 // iinc
        0x85 => Entry::plain(POP_I, PUSH_L),                    // i2l
        0x86 => Entry::plain(POP_I, PUSH_F),                    // i2f
        0x87 => Entry::plain(POP_I, PUSH_D),                    // i2d
        0x88 => Entry::plain(POP_L, PUSH_I),                    // l2i
        0x89 => Entry::plain(POP_L, PUSH_F),                    // l2f
        0x8a => Entry::plain(POP_L, PUSH_D),                    // l2d
        0x8b => Entry::plain(POP_F, PUSH_I),                    // f2i
        0x8c => Entry::plain(POP_F, PUSH_L),                    // f2l
        0x8d => Entry::plain(POP_F, PUSH_D),                    // f2d
        0x8e => Entry::plain(POP_D, PUSH_I),                    // d2i
        0x8f => Entry::plain(POP_D, PUSH_L),                    // d2l
        0x90 => Entry::plain(POP_D, PUSH_F),                    // d2f
        0x91..=0x93 => Entry::plain(POP_I, PUSH_I),             // i2b, i2c, i2s
        0x94 => Entry::plain(POP_LL, PUSH_I),                   // lcmp
        0x95 | 0x96 => Entry::plain(POP_FF, PUSH_I),            // fcmpl, fcmpg
        0x97 | 0x98 => Entry::plain(POP_DD, PUSH_I),            // dcmpl, dcmpg
        0x99..=0x9e => Entry::plain(POP_I, PUSH_NONE).ends(),   // ifeq..ifle
        0x9f..=0xa4 => Entry::plain(POP_II, PUSH_NONE).ends(),  // if_icmpeq..if_icmple
        // `if_acmpeq`/`if_acmpne` are the two comparisons of *references*: the opcode
        // range they continue is not a range of the same shape, which is why they have
        // their own row rather than following the integer forms.
        0xa5 | 0xa6 => Entry::plain(POP_RR, PUSH_NONE).ends(), // if_acmpeq, if_acmpne
        0xa7 => Entry::plain(POP_NONE, PUSH_NONE).ends(),      // goto
        0xa8 => Entry::plain(POP_NONE, PUSH_RET).ends(),       // jsr
        0xa9 => Entry::local(Local::Return).ends(),            // ret
        0xaa | 0xab => Entry::plain(POP_I, PUSH_NONE).ends(),  // tableswitch, lookupswitch
        0xac => Entry::plain(POP_I, PUSH_NONE).ends(),         // ireturn
        0xad => Entry::plain(POP_L, PUSH_NONE).ends(),         // lreturn
        0xae => Entry::plain(POP_F, PUSH_NONE).ends(),         // freturn
        0xaf => Entry::plain(POP_D, PUSH_NONE).ends(),         // dreturn
        0xb0 => Entry::plain(POP_R, PUSH_NONE).ends(),         // areturn
        0xb1 => Entry::plain(POP_NONE, PUSH_NONE).ends(),      // return
        0xb2 => Entry::constant(PoolEffect::Field {
            write: false,
            target: false,
        }), // getstatic
        0xb3 => Entry::constant(PoolEffect::Field {
            write: true,
            target: false,
        }), // putstatic
        0xb4 => Entry::constant(PoolEffect::Field {
            write: false,
            target: true,
        }), // getfield
        0xb5 => Entry::constant(PoolEffect::Field {
            write: true,
            target: true,
        }), // putfield
        0xb6 | 0xb7 | 0xb9 => Entry::constant(PoolEffect::Invoke { receiver: true }),
        0xb8 | 0xba => Entry::constant(PoolEffect::Invoke { receiver: false }),
        0xbb => Entry::constant(PoolEffect::New), // new
        0xbc => Entry::constant(PoolEffect::NewArray), // newarray
        0xbd => Entry::constant(PoolEffect::ANewArray), // anewarray
        0xbe => Entry::plain(POP_R, PUSH_I),      // arraylength
        0xbf => Entry::constant(PoolEffect::AThrow).ends(), // athrow
        0xc0 => Entry::constant(PoolEffect::CheckCast), // checkcast
        0xc1 => Entry::plain(POP_R, PUSH_I),      // instanceof
        0xc2 | 0xc3 => Entry::plain(POP_R, PUSH_NONE), // monitorenter, monitorexit
        0xc5 => Entry::constant(PoolEffect::MultiANewArray), // multianewarray
        0xc6 | 0xc7 => Entry::plain(POP_R, PUSH_NONE).ends(), // ifnull, ifnonnull
        0xc8 => Entry::plain(POP_NONE, PUSH_NONE).ends(), // goto_w
        0xc9 => Entry::plain(POP_NONE, PUSH_RET).ends(), // jsr_w
        // `0xc4` is the `wide` prefix and never reaches this table (the operands' effective
        // opcode is the wrapped one), and everything from `0xca` up is reserved, a breakpoint or
        // an implementation-dependent instruction no decoded body carries.
        _ => Entry::NOT_AN_OPCODE,
    }
}

/// One phase of a run of this pass, named where [`checkpoint`] is called.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    /// The entry frame of the method.
    Entry,
    /// One block of the fixpoint: its transfer and the merge into its successors.
    Transfer,
    /// The assembly of the published table.
    Publish,
}

/// Polls the run's own stop condition at one phase of the pass.
///
/// Every phase either charges (`Budget::charge` polls by itself) or reaches this, so a cancelled
/// or exhausted run stops at a phase boundary instead of finishing the table. The stop is the
/// ordinary `Error::Cancelled`/`Error::BudgetExceeded` the rest of the crate reports, and a
/// stopped run publishes no [`FrameTable`].
fn checkpoint(phase: Phase, budget: &Budget) -> Result<()> {
    #[cfg(test)]
    if CHECKPOINT.with(|slot| slot.get()) == Some(phase) {
        CHECKPOINT.with(|slot| slot.set(None));
        budget.cancellation_token().cancel();
    }
    let _ = phase;
    budget.poll()
}

#[cfg(test)]
thread_local! {
    /// The phase a test cancels at, and whether the checkpoint has already fired.
    static CHECKPOINT: std::cell::Cell<Option<Phase>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn checkpoint_seam(phase: Phase) -> CheckpointSeam {
    CHECKPOINT.with(|slot| slot.set(Some(phase)));
    CheckpointSeam
}

/// Clears one test's seam when the test ends, so the next test starts inert.
#[cfg(test)]
struct CheckpointSeam;

#[cfg(test)]
impl Drop for CheckpointSeam {
    fn drop(&mut self) {
        CHECKPOINT.with(|slot| slot.set(None));
    }
}

/// The three ways one run of this pass can end.
///
/// The `Unproven`/`Inconsistent` split is the point of this type: the first is a gap of this
/// build that 4.2 closes, the second is something the method's own bytes decide. A run that gave
/// up on an uninitialized value must never be reported as the second, and a body that contradicts
/// itself must never be excused as the first.
enum Problem {
    /// A stop the budget layer raised: a cancellation or an exhausted counted dimension.
    Budget(Error),
    /// The body contradicts itself: [`IR_FRAME_INCONSISTENT`].
    Inconsistent(String),
    /// This build does not prove the state yet: [`IR_FRAME_DEFERRED`].
    Unproven(String),
}

/// The outcome of one step of this pass: the table, or why the step could not produce one.
type Norm<T> = std::result::Result<T, Problem>;

impl From<Error> for Problem {
    fn from(error: Error) -> Self {
        Self::Budget(error)
    }
}

/// Reports a contradiction of the body.
fn inconsistent<T>(message: String) -> Norm<T> {
    Err(Problem::Inconsistent(message))
}

/// One frame state: the local slots and the operand stack, both by slot.
///
/// A category-2 value occupies two entries of a frame — the value and [`Value::Second`] — so every
/// index in this module is a **slot** index, and the depth of the stack is its length.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Frame {
    /// One entry per local slot, index 0 first.
    locals: Vec<Value>,
    /// The operand stack, bottom first.
    stack: Vec<Value>,
}

impl Frame {
    /// Pushes one value, marking the upper slot of a category-2 one.
    ///
    /// The JVM's slot ceiling is checked here and not at the end of the block: a body whose stack
    /// would grow past it contradicts itself, and the check is also what keeps one block's
    /// derived state finite while the worklist runs.
    fn push(&mut self, value: Value, bci: u32) -> Norm<()> {
        let depth = self.stack.len() + value.slots();
        if depth > MAX_STACK_SLOTS {
            return inconsistent(format!(
                "the operand stack at BCI {bci} would hold {depth} slots, above the JVM's ceiling \
                 of {MAX_STACK_SLOTS}"
            ));
        }
        if value.slots() == 2 {
            self.stack.push(value);
            self.stack.push(Value::Second);
        } else {
            self.stack.push(value);
        }
        Ok(())
    }

    /// Removes the top value, category-2 values included.
    ///
    /// A slot that holds no readable value cannot be taken: `Top` means "not readable", and the
    /// upper slot of a category-2 value belongs to the value below it, so neither may be read on
    /// its own.
    fn take(&mut self, bci: u32, opcode: u8) -> Norm<Value> {
        let Some(top) = self.stack.pop() else {
            return inconsistent(format!(
                "the operand stack at BCI {bci} is empty where `{opcode:#04x}` takes a value"
            ));
        };
        match top {
            Value::Second => {
                let Some(first) = self.stack.pop() else {
                    return inconsistent(format!(
                        "BCI {bci} holds an upper category-2 slot with no slot below it"
                    ));
                };
                match first {
                    Value::Long | Value::Double => Ok(first),
                    other => inconsistent(format!(
                        "the upper slot at BCI {bci} belongs to {other:?}, not to a category-2 \
                         value"
                    )),
                }
            }
            Value::Long | Value::Double => inconsistent(format!(
                "the category-2 value at BCI {bci} is missing its upper slot"
            )),
            Value::Top => inconsistent(format!(
                "the operand stack at BCI {bci} holds no readable value"
            )),
            value => Ok(value),
        }
    }

    /// Removes the top value and requires it to be a category-1 computation type — the shape the
    /// `dup*`/`swap` forms call `value1`, `value2`, ... .
    fn take_cat1(&mut self, bci: u32, opcode: u8) -> Norm<Value> {
        let value = self.take(bci, opcode)?;
        if value.slots() == 2 {
            return inconsistent(format!(
                "`{opcode:#04x}` at BCI {bci} needs a category-1 value where the stack holds a \
                 category-2 one"
            ));
        }
        Ok(value)
    }

    /// Removes the top value and requires it to fit one slot class.
    ///
    /// `moves` says whether the instruction is one of the pure stack operations
    /// (`astore`/`aload`, `dup*`, `pop*`, `swap`): those carry an uninitialized value without
    /// interpreting it. Every other consumer of one stops the body — 4.2's initialization
    /// analysis is what would make that state usable — so the run neither guesses nor calls the
    /// body inconsistent.
    fn pop_ty(&mut self, ty: Ty, bci: u32, opcode: u8, moves: bool) -> Norm<Value> {
        let value = self.take(bci, opcode)?;
        if value.is_uninitialized() {
            if !moves || ty != Ty::Ref {
                return Err(Problem::Unproven(format!(
                    "`{opcode:#04x}` at BCI {bci} consumes the uninitialized value {value:?} as \
                     something other than a moved reference; the initialization conversions of \
                     4.2 are what would make this state usable"
                )));
            }
            return Ok(value);
        }
        if !ty.accepts(&value) {
            return inconsistent(format!(
                "`{opcode:#04x}` at BCI {bci} needs a {ty:?} where the stack holds {value:?}"
            ));
        }
        Ok(value)
    }

    /// Reads the local the operands name, which must hold a value of this slot class, and returns
    /// the local's own value so a caller can push it unchanged.
    fn read_local(&mut self, ty: Ty, index: u16, bci: u32, opcode: u8) -> Norm<Value> {
        let Some(value) = self.locals.get(usize::from(index)).cloned() else {
            return inconsistent(format!(
                "`{opcode:#04x}` at BCI {bci} reads local {index}, which the frame of {} slots \
                 does not hold",
                self.locals.len()
            ));
        };
        match value {
            Value::Top | Value::Second => inconsistent(format!(
                "`{opcode:#04x}` at BCI {bci} reads local {index}, which holds no readable value"
            )),
            Value::UninitializedThis | Value::Uninitialized { .. } if ty == Ty::Ref => Ok(value),
            Value::UninitializedThis | Value::Uninitialized { .. } => {
                Err(Problem::Unproven(format!(
                    "`{opcode:#04x}` at BCI {bci} reads the uninitialized value {value:?} from local \
                 {index} as a {ty:?}; the initialization conversions of 4.2 are what would make \
                 this state usable"
                )))
            }
            value if ty.accepts(&value) => Ok(value),
            value => inconsistent(format!(
                "`{opcode:#04x}` at BCI {bci} reads local {index} as a {ty:?} where it holds \
                 {value:?}"
            )),
        }
    }

    /// Writes one value into the local the operands name, invalidating a category-2 pair it
    /// covers on either half.
    ///
    /// Covering the **upper** slot of a pair removes the value below it, and covering the **lower**
    /// slot removes the upper one: the two slots were one value, and the half that is left no
    /// longer belongs to it. Both directions end in `Top`, never in a surviving half.
    fn write_local(&mut self, index: u16, value: Value, bci: u32) -> Norm<()> {
        let start = usize::from(index);
        let end = start + value.slots();
        if end > self.locals.len() {
            return inconsistent(format!(
                "BCI {bci} writes a {}-slot value into local {index}, past the {} slots the frame \
                 holds",
                value.slots(),
                self.locals.len()
            ));
        }
        for slot in start..end {
            match self.locals[slot] {
                Value::Second => match slot.checked_sub(1) {
                    Some(lower) => self.locals[lower] = Value::Top,
                    None => {
                        return inconsistent(format!(
                            "the first local slot holds an upper category-2 slot at BCI {bci}, \
                             which no store can produce"
                        ));
                    }
                },
                Value::Long | Value::Double => {
                    if let Some(upper) = self.locals.get_mut(slot + 1) {
                        *upper = Value::Top;
                    }
                }
                _ => {}
            }
        }
        self.locals[start] = value;
        if end > start + 1 {
            self.locals[start + 1] = Value::Second;
        }
        Ok(())
    }
}

/// The merge of two states of one local slot.
///
/// A disagreement is not a refusal here, and that is the rule the design fixes: a local only two
/// paths disagree about is a local no read may use, so the merge answers `Top` and the failure
/// moves to the read. References are the one family with a real least upper bound available
/// without a class hierarchy: two different names merge to a conservative unknown reference,
/// `null` merges with a named reference into that reference, and the null type with itself stays
/// the null type.
fn merge_local(left: &Value, right: &Value) -> Value {
    if left == right {
        return left.clone();
    }
    match (left, right) {
        (Value::Top, _) | (_, Value::Top) | (Value::Second, _) | (_, Value::Second) => Value::Top,
        (Value::Null, Value::Ref(other)) | (Value::Ref(other), Value::Null) => {
            Value::Ref(other.clone())
        }
        (Value::Ref(left), Value::Ref(right)) => Value::Ref(if left == right {
            left.clone()
        } else {
            RefType::Unknown
        }),
        _ => Value::Top,
    }
}

/// The merge of two operand stacks entering one block.
///
/// The stack is the half that must agree: depth and slot classes decide whether the body can hold
/// a value at all, so a disagreement here is `ir_frame_inconsistent` and not a `Top`.
/// Uninitialized values are the one case this build does not decide — merging one with a different
/// state is exactly the alias conversion 4.2 owns — so they stop the body under their own code
/// instead of being reported as a contradiction.
fn merge_stack(left: &[Value], right: &[Value], block: &CanonicalBlockId) -> Norm<Vec<Value>> {
    if left.len() != right.len() {
        return inconsistent(format!(
            "block {block:?} is entered with operand stacks of {} and {} slots",
            left.len(),
            right.len()
        ));
    }
    let mut merged = Vec::with_capacity(left.len());
    for (slot, (left, right)) in left.iter().zip(right.iter()).enumerate() {
        if left == right {
            merged.push(left.clone());
            continue;
        }
        match (left, right) {
            (Value::Null, Value::Ref(other)) | (Value::Ref(other), Value::Null) => {
                merged.push(Value::Ref(other.clone()));
            }
            (Value::Ref(left), Value::Ref(right)) => {
                merged.push(Value::Ref(if left == right {
                    left.clone()
                } else {
                    RefType::Unknown
                }));
            }
            (Value::UninitializedThis | Value::Uninitialized { .. }, _)
            | (_, Value::UninitializedThis | Value::Uninitialized { .. }) => {
                return Err(Problem::Unproven(format!(
                    "slot {slot} of block {block:?} merges the uninitialized value {left:?} with \
                     {right:?}; the alias conversion of 4.2 is what would decide that state"
                )));
            }
            (left, right) => {
                return inconsistent(format!(
                    "slot {slot} of block {block:?} merges the stack values {left:?} and {right:?}"
                ));
            }
        }
    }
    Ok(merged)
}

/// The merge of two frames entering one block: the locals under the local rule, the stack under
/// the stack rule.
fn merge_frame(current: &Frame, incoming: &Frame, block: &CanonicalBlockId) -> Norm<Frame> {
    if current.locals.len() != incoming.locals.len() {
        return inconsistent(format!(
            "block {block:?} is entered with frames of {} and {} local slots",
            current.locals.len(),
            incoming.locals.len()
        ));
    }
    let locals = current
        .locals
        .iter()
        .zip(incoming.locals.iter())
        .map(|(left, right)| merge_local(left, right))
        .collect();
    let stack = merge_stack(&current.stack, &incoming.stack, block)?;
    Ok(Frame { locals, stack })
}

/// One parameter of a method descriptor: its slot class, and the type the descriptor spells for a
/// reference.
struct Param {
    /// The slot class the parameter occupies.
    ty: Ty,
    /// The internal name the descriptor spells, for a reference parameter.
    name: Option<Vec<u8>>,
}

/// Parses one field descriptor at the start of `bytes`.
///
/// Returns the slot class, the number of bytes the descriptor used, and — for a reference — the
/// internal name the descriptor spells, which is what [`RefType::Named`] carries. A descriptor the
/// class-file format does not allow is a contradiction of the body: the reader keeps the bytes and
/// does not validate them, so this is where a malformed one is refused.
fn parse_field_type(bytes: &[u8]) -> Norm<(Ty, usize, Option<Vec<u8>>)> {
    let Some((&first, rest)) = bytes.split_first() else {
        return inconsistent("a field descriptor is empty".to_string());
    };
    Ok(match first {
        b'B' | b'C' | b'I' | b'S' | b'Z' => (Ty::Int, 1, None),
        b'F' => (Ty::Float, 1, None),
        b'J' => (Ty::Long, 1, None),
        b'D' => (Ty::Double, 1, None),
        b'L' => {
            let Some(semicolon) = rest.iter().position(|byte| *byte == b';') else {
                return inconsistent(format!(
                    "the field descriptor `{}` is a class type without its `;`",
                    String::from_utf8_lossy(bytes)
                ));
            };
            let length = semicolon + 2;
            (Ty::Ref, length, Some(bytes[..length].to_vec()))
        }
        b'[' => {
            let (_, element, _) = parse_field_type(rest)?;
            let length = element + 1;
            (Ty::Ref, length, Some(bytes[..length].to_vec()))
        }
        _ => {
            return inconsistent(format!(
                "`{}` is not a field descriptor",
                String::from_utf8_lossy(bytes)
            ));
        }
    })
}

/// Parses a method descriptor `(parameters)return`.
fn parse_method_descriptor(bytes: &[u8]) -> Norm<(Vec<Param>, Option<Param>)> {
    let Some((&b'(', rest)) = bytes.split_first() else {
        return inconsistent(format!(
            "`{}` is not a method descriptor",
            String::from_utf8_lossy(bytes)
        ));
    };
    let mut params = Vec::new();
    let mut offset = 0usize;
    loop {
        match rest.get(offset) {
            Some(b')') => {
                offset += 1;
                break;
            }
            Some(_) => {
                let (ty, length, name) = parse_field_type(&rest[offset..])?;
                params.push(Param { ty, name });
                offset += length;
            }
            None => {
                return inconsistent(format!(
                    "the method descriptor `{}` is not terminated by `)`",
                    String::from_utf8_lossy(bytes)
                ));
            }
        }
    }
    let returns = match rest.get(offset) {
        Some(b'V') if offset + 1 == rest.len() => None,
        Some(_) => {
            let (ty, length, name) = parse_field_type(&rest[offset..])?;
            if offset + length != rest.len() {
                return inconsistent(format!(
                    "the method descriptor `{}` has bytes after its return type",
                    String::from_utf8_lossy(bytes)
                ));
            }
            Some(Param { ty, name })
        }
        None => {
            return inconsistent(format!(
                "the method descriptor `{}` names no return type",
                String::from_utf8_lossy(bytes)
            ));
        }
    };
    Ok((params, returns))
}

/// The value one slot class produces, named where the class file names the type.
fn value_of(ty: Ty, name: Option<Vec<u8>>, method: &FrameMethod<'_>) -> Value {
    match ty {
        Ty::Int => Value::Int,
        Ty::Float => Value::Float,
        Ty::Long => Value::Long,
        Ty::Double => Value::Double,
        Ty::Ref => match name {
            Some(name) => named(name, method),
            None => Value::Ref(RefType::Unknown),
        },
    }
}

/// A reference whose type the class file names, anchored to the request's loader.
fn named(name: Vec<u8>, method: &FrameMethod<'_>) -> Value {
    Value::Ref(RefType::Named {
        name,
        loader: Box::new(method.loader.clone()),
    })
}

/// The name of one constant-pool kind, for the diagnostic of an operand that names the wrong one.
fn cp_kind_name(kind: &CpEntryKind) -> &'static str {
    match kind {
        CpEntryKind::Utf8 { .. } => "Utf8",
        CpEntryKind::Integer { .. } => "Integer",
        CpEntryKind::Float { .. } => "Float",
        CpEntryKind::Long { .. } => "Long",
        CpEntryKind::Double { .. } => "Double",
        CpEntryKind::Class { .. } => "Class",
        CpEntryKind::String { .. } => "String",
        CpEntryKind::FieldRef { .. } => "Fieldref",
        CpEntryKind::MethodRef { .. } => "Methodref",
        CpEntryKind::InterfaceMethodRef { .. } => "InterfaceMethodref",
        CpEntryKind::NameAndType { .. } => "NameAndType",
        CpEntryKind::MethodHandle { .. } => "MethodHandle",
        CpEntryKind::MethodType { .. } => "MethodType",
        CpEntryKind::Dynamic { .. } => "Dynamic",
        CpEntryKind::InvokeDynamic { .. } => "InvokeDynamic",
        CpEntryKind::Module { .. } => "Module",
        CpEntryKind::Package { .. } => "Package",
    }
}

/// The constant-pool entry one instruction names, or a structured refusal.
fn pool_entry<'a>(
    pool: &'a [CpEntryFacts],
    operands: &InstructionOperands,
    bci: u32,
    opcode: u8,
) -> Norm<&'a CpEntryFacts> {
    let Some(index) = operands.constant_pool_index else {
        return inconsistent(format!(
            "`{opcode:#04x}` at BCI {bci} names no constant-pool entry, which its encoding requires"
        ));
    };
    jarde_reader::classfile::cp_entry(pool, index).map_err(|error| {
        Problem::Inconsistent(format!(
            "`{opcode:#04x}` at BCI {bci} names the constant-pool entry {index}: {error}"
        ))
    })
}

/// The internal name one `Class` operand names.
fn pool_class_name(
    method: &FrameMethod<'_>,
    operands: &InstructionOperands,
    bci: u32,
    opcode: u8,
) -> Norm<Vec<u8>> {
    let entry = pool_entry(method.pool, operands, bci, opcode)?;
    match &entry.kind {
        CpEntryKind::Class { name, .. } => Ok(name.0.clone()),
        other => inconsistent(format!(
            "`{opcode:#04x}` at BCI {bci} names a {} entry where the format requires a class type",
            cp_kind_name(other)
        )),
    }
}

/// The field descriptor one `Fieldref` operand names.
fn pool_field_descriptor(
    method: &FrameMethod<'_>,
    operands: &InstructionOperands,
    bci: u32,
    opcode: u8,
) -> Norm<Vec<u8>> {
    let entry = pool_entry(method.pool, operands, bci, opcode)?;
    match &entry.kind {
        CpEntryKind::FieldRef { descriptor, .. } => Ok(descriptor.0.clone()),
        other => inconsistent(format!(
            "`{opcode:#04x}` at BCI {bci} names a {} entry where a field reference is required",
            cp_kind_name(other)
        )),
    }
}

/// The method descriptor one invocation operand names.
///
/// The opcode decides which constant-pool kinds may carry the method it invokes, and this layer
/// checks that pairing for the same reason it checks it on the field side
/// ([`pool_field_descriptor`] takes a `Fieldref` and nothing else): a descriptor read out of an
/// entry the instruction may not name is a shape derived from the wrong fact. Doing half of the
/// pairing would leave "`getfield` over a `Methodref` contradicts the bytes" standing next to
/// "`invokevirtual` over an `InvokeDynamic` is believed", and a reader could not tell which half
/// was intended. The conservative direction is both halves — an illegal input is named as what it
/// is instead of being given a shape. This is not the verifier: the check is on the kind of the
/// entry, and says nothing else about it.
fn pool_method_descriptor(
    method: &FrameMethod<'_>,
    operands: &InstructionOperands,
    bci: u32,
    opcode: u8,
) -> Norm<Vec<u8>> {
    let entry = pool_entry(method.pool, operands, bci, opcode)?;
    match (&entry.kind, opcode) {
        // The pairing JVMS 6.5 and SE 8 define, opcode by opcode: a class method through
        // `0xb6`/`0xb7`/`0xb8`, an interface method through `0xb7`/`0xb8`/`0xb9` (the two opcodes
        // in both ranges are the ones an interface's own methods reach since SE 8), and the one
        // dynamic call site of `0xba`. `invocation_entries` names the same five opcodes in words
        // for the refusal.
        (CpEntryKind::MethodRef { descriptor, .. }, 0xb6..=0xb8)
        | (CpEntryKind::InterfaceMethodRef { descriptor, .. }, 0xb7..=0xb9)
        | (CpEntryKind::InvokeDynamic { descriptor, .. }, 0xba) => Ok(descriptor.0.clone()),
        (kind, _) => inconsistent(format!(
            "`{opcode:#04x}` at BCI {bci} names a {} entry where {} is required",
            cp_kind_name(kind),
            invocation_entries(opcode)
        )),
    }
}

/// The constant-pool kinds one invocation opcode may name, as the refusal says them.
///
/// `invokevirtual` is the call of a class method. Since SE 8 the two other opcodes that take one
/// may name an interface method as well — `invokespecial` reaches an interface's default and
/// private methods, `invokestatic` its static ones, and both of those are a
/// `CONSTANT_InterfaceMethodref` — while `invokeinterface` and `invokedynamic` each name their own
/// kind. `invokedynamic` is the one opcode that consumes a `CONSTANT_InvokeDynamic`.
fn invocation_entries(opcode: u8) -> &'static str {
    match opcode {
        0xb6 => "a Methodref",
        0xb7 | 0xb8 => "a Methodref or an InterfaceMethodref",
        0xb9 => "an InterfaceMethodref",
        0xba => "an InvokeDynamic",
        _ => "a method reference",
    }
}

/// The local index one instruction's operands name.
fn local_index(operands: &InstructionOperands, bci: u32, opcode: u8) -> Norm<u16> {
    match operands.local {
        Some(local) => Ok(local.index),
        None => inconsistent(format!(
            "`{opcode:#04x}` at BCI {bci} names no local, which its encoding requires"
        )),
    }
}

/// The array type one element type code names, as `newarray` pushes it.
fn primitive_array_of(atype: u8) -> Option<&'static [u8]> {
    Some(match atype {
        4 => b"[Z",
        5 => b"[C",
        6 => b"[F",
        7 => b"[D",
        8 => b"[B",
        9 => b"[S",
        10 => b"[I",
        11 => b"[J",
        _ => return None,
    })
}

/// The array type of one class name, as `anewarray` forms it.
fn array_of(name: &[u8]) -> Vec<u8> {
    let mut array = Vec::with_capacity(name.len() + 2);
    array.push(b'[');
    if name.first() == Some(&b'[') {
        array.extend_from_slice(name);
    } else {
        array.push(b'L');
        array.extend_from_slice(name);
        array.push(b';');
    }
    array
}

/// Applies one instruction to a frame.
///
/// The three families are the three ways an instruction can have a shape: the opcode decides it,
/// the operand stack decides it, or the class file decides it. A local access is applied here and
/// not by the table row's stack sequence, because the value it moves is the local's own: whether a
/// load pushes an uninitialized value or a return address is a fact of the slot, not of the
/// opcode.
fn apply(
    method: &FrameMethod<'_>,
    entry: Entry,
    bci: u32,
    operands: &InstructionOperands,
    frame: &mut Frame,
) -> Norm<()> {
    let opcode = operands.effective_opcode;
    // The pure stack operations move an uninitialized value; every other row interprets what it
    // takes, which is where 4.1 has to stop instead of guessing.
    let moves =
        matches!(entry.stack, Stack::Form(_)) || matches!(entry.local, Local::Store(Ty::Ref));
    match entry.stack {
        Stack::NotAnOpcode => inconsistent(format!(
            "opcode {opcode:#04x} at BCI {bci} is not an instruction of a decodable body"
        )),
        Stack::Form(form) => apply_form(form, bci, opcode, frame),
        Stack::Fixed { pops, pushes } => {
            let loaded = match entry.local {
                Local::Load(ty) => {
                    Some(frame.read_local(ty, local_index(operands, bci, opcode)?, bci, opcode)?)
                }
                _ => None,
            };
            for ty in pops {
                frame.pop_ty(*ty, bci, opcode, moves)?;
            }
            match entry.local {
                Local::Store(ty) => {
                    let value = frame.pop_ty(ty, bci, opcode, moves)?;
                    frame.write_local(local_index(operands, bci, opcode)?, value, bci)?;
                }
                Local::Increment => {
                    // `iinc` needs an `int` in the slot and leaves an `int` there, and its
                    // category-2 invalidation is the store rule like any other write.
                    let index = local_index(operands, bci, opcode)?;
                    frame.read_local(Ty::Int, index, bci, opcode)?;
                    frame.write_local(index, Value::Int, bci)?;
                }
                Local::Return => {
                    let index = local_index(operands, bci, opcode)?;
                    let value = frame.read_local(Ty::Ref, index, bci, opcode)?;
                    if value != Value::ReturnAddress {
                        return inconsistent(format!(
                            "`ret` at BCI {bci} returns through local {index}, which holds \
                             {value:?} and not a return address"
                        ));
                    }
                }
                Local::None | Local::Load(_) => {}
            }
            for produced in pushes {
                frame.push(produced.value(None), bci)?;
            }
            if let Some(value) = loaded {
                frame.push(value, bci)?;
            }
            Ok(())
        }
        Stack::Constant(effect) => apply_constant(method, effect, bci, operands, frame),
    }
}

/// Applies one of the `dup*`/`pop*`/`swap` forms.
///
/// Every form names its JVMS pattern explicitly, and a shape the instruction may not see is refused
/// with the value that broke it. That is what makes `dup2` of two category-1 values and `dup2` of
/// one category-2 value two different cases of one instruction instead of one shape that happens
/// to be right for both.
fn apply_form(form: Form, bci: u32, opcode: u8, frame: &mut Frame) -> Norm<()> {
    match form {
        Form::Pop => {
            frame.take(bci, opcode)?;
        }
        Form::Pop2 => {
            // `pop2` takes one category-2 value or two category-1 ones.
            let first = frame.take(bci, opcode)?;
            if first.slots() == 1 {
                frame.take_cat1(bci, opcode)?;
            }
        }
        Form::Dup => {
            // `..., value1` -> `..., value1, value1`.
            let value = frame.take_cat1(bci, opcode)?;
            frame.push(value.clone(), bci)?;
            frame.push(value, bci)?;
        }
        Form::DupX1 => {
            // `..., value2, value1` -> `..., value1, value2, value1`.
            let value1 = frame.take_cat1(bci, opcode)?;
            let value2 = frame.take_cat1(bci, opcode)?;
            frame.push(value1.clone(), bci)?;
            frame.push(value2, bci)?;
            frame.push(value1, bci)?;
        }
        Form::DupX2 => {
            // `..., value3, value2, value1` -> `..., value1, value3, value2, value1`, or with a
            // category-2 `value2`: `..., value2, value1` -> `..., value1, value2, value1`.
            let value1 = frame.take_cat1(bci, opcode)?;
            let below = frame.take(bci, opcode)?;
            if below.slots() == 2 {
                frame.push(value1.clone(), bci)?;
                frame.push(below, bci)?;
                frame.push(value1, bci)?;
            } else {
                let value3 = frame.take_cat1(bci, opcode)?;
                frame.push(value1.clone(), bci)?;
                frame.push(value3, bci)?;
                frame.push(below, bci)?;
                frame.push(value1, bci)?;
            }
        }
        Form::Dup2 => {
            // Two category-1 values, or one category-2 value.
            let value1 = frame.take(bci, opcode)?;
            if value1.slots() == 2 {
                frame.push(value1.clone(), bci)?;
                frame.push(value1, bci)?;
            } else {
                let value2 = frame.take_cat1(bci, opcode)?;
                frame.push(value2.clone(), bci)?;
                frame.push(value1.clone(), bci)?;
                frame.push(value2, bci)?;
                frame.push(value1, bci)?;
            }
        }
        Form::Dup2X1 => {
            // Two category-1 values over one category-1 value, or one category-2 value over one
            // category-1 value.
            let value1 = frame.take(bci, opcode)?;
            if value1.slots() == 2 {
                let value2 = frame.take_cat1(bci, opcode)?;
                frame.push(value1.clone(), bci)?;
                frame.push(value2, bci)?;
                frame.push(value1, bci)?;
            } else {
                let value2 = frame.take_cat1(bci, opcode)?;
                let value3 = frame.take_cat1(bci, opcode)?;
                frame.push(value2.clone(), bci)?;
                frame.push(value1.clone(), bci)?;
                frame.push(value3, bci)?;
                frame.push(value2, bci)?;
                frame.push(value1, bci)?;
            }
        }
        Form::Dup2X2 => {
            // The four forms of JVMS 6.5: two category-1 values over two, one category-1 value
            // over a category-2 one over a category-1 one, a category-2 value over two
            // category-1 ones, and one category-2 value over one category-1 one. Two category-2
            // values below each other are **not** a form of this instruction.
            let value1 = frame.take(bci, opcode)?;
            if value1.slots() == 2 {
                let value2 = frame.take_cat1(bci, opcode)?;
                frame.push(value1.clone(), bci)?;
                frame.push(value2, bci)?;
                frame.push(value1, bci)?;
            } else {
                let value2 = frame.take(bci, opcode)?;
                if value2.slots() == 2 {
                    let value3 = frame.take_cat1(bci, opcode)?;
                    frame.push(value2.clone(), bci)?;
                    frame.push(value1.clone(), bci)?;
                    frame.push(value3, bci)?;
                    frame.push(value2, bci)?;
                    frame.push(value1, bci)?;
                } else {
                    let value3 = frame.take(bci, opcode)?;
                    if value3.slots() == 2 {
                        frame.push(value2.clone(), bci)?;
                        frame.push(value1.clone(), bci)?;
                        frame.push(value3, bci)?;
                        frame.push(value2, bci)?;
                        frame.push(value1, bci)?;
                    } else {
                        let value4 = frame.take_cat1(bci, opcode)?;
                        frame.push(value2.clone(), bci)?;
                        frame.push(value1.clone(), bci)?;
                        frame.push(value4, bci)?;
                        frame.push(value3, bci)?;
                        frame.push(value2, bci)?;
                        frame.push(value1, bci)?;
                    }
                }
            }
        }
        Form::Swap => {
            // Two category-1 values, exchanged; a category-2 value is not a shape `swap` has.
            let value1 = frame.take_cat1(bci, opcode)?;
            let value2 = frame.take_cat1(bci, opcode)?;
            frame.push(value1, bci)?;
            frame.push(value2, bci)?;
        }
    }
    Ok(())
}

/// Applies one instruction whose shape the class file decides.
fn apply_constant(
    method: &FrameMethod<'_>,
    effect: PoolEffect,
    bci: u32,
    operands: &InstructionOperands,
    frame: &mut Frame,
) -> Norm<()> {
    let opcode = operands.effective_opcode;
    match effect {
        PoolEffect::Ldc => {
            let entry = pool_entry(method.pool, operands, bci, opcode)?;
            let value = match &entry.kind {
                CpEntryKind::Integer { .. } => Value::Int,
                CpEntryKind::Float { .. } => Value::Float,
                // The instruction names no class, so the reference stays unknown: a `String`,
                // `Class`, `MethodType` or `MethodHandle` constant is a reference whose type
                // these facts do not spell out.
                CpEntryKind::String { .. }
                | CpEntryKind::Class { .. }
                | CpEntryKind::MethodType { .. }
                | CpEntryKind::MethodHandle { .. } => Value::Ref(RefType::Unknown),
                CpEntryKind::Dynamic { descriptor, .. } => {
                    let (ty, _, name) = parse_field_type(&descriptor.0)?;
                    if ty.slots() == 2 {
                        return inconsistent(format!(
                            "`ldc` at BCI {bci} names the category-2 dynamic constant `{}`, which \
                             only `ldc2_w` may load",
                            String::from_utf8_lossy(&descriptor.0)
                        ));
                    }
                    value_of(ty, name, method)
                }
                other => {
                    return inconsistent(format!(
                        "`{opcode:#04x}` at BCI {bci} loads a {} constant, whose value this \
                         instruction cannot carry",
                        cp_kind_name(other)
                    ));
                }
            };
            frame.push(value, bci)
        }
        PoolEffect::Ldc2 => {
            let entry = pool_entry(method.pool, operands, bci, opcode)?;
            let value = match &entry.kind {
                CpEntryKind::Long { .. } => Value::Long,
                CpEntryKind::Double { .. } => Value::Double,
                CpEntryKind::Dynamic { descriptor, .. } => {
                    let (ty, _, name) = parse_field_type(&descriptor.0)?;
                    if ty.slots() != 2 {
                        return inconsistent(format!(
                            "`ldc2_w` at BCI {bci} names the dynamic constant `{}`, which is not \
                             a category-2 value",
                            String::from_utf8_lossy(&descriptor.0)
                        ));
                    }
                    value_of(ty, name, method)
                }
                other => {
                    return inconsistent(format!(
                        "`{opcode:#04x}` at BCI {bci} loads a {} constant where the format \
                         requires a category-2 one",
                        cp_kind_name(other)
                    ));
                }
            };
            frame.push(value, bci)
        }
        PoolEffect::Field { write, target } => {
            let descriptor = pool_field_descriptor(method, operands, bci, opcode)?;
            let (ty, length, name) = parse_field_type(&descriptor)?;
            if length != descriptor.len() {
                return inconsistent(format!(
                    "the field descriptor `{}` of BCI {bci} has bytes after its field type",
                    String::from_utf8_lossy(&descriptor)
                ));
            }
            if write {
                // `putfield` consumes the value over the target; `putstatic` the value alone.
                frame.pop_ty(ty, bci, opcode, false)?;
                if target {
                    frame.pop_ty(Ty::Ref, bci, opcode, false)?;
                }
                Ok(())
            } else {
                if target {
                    frame.pop_ty(Ty::Ref, bci, opcode, false)?;
                }
                frame.push(value_of(ty, name, method), bci)
            }
        }
        PoolEffect::Invoke { receiver } => {
            let descriptor = pool_method_descriptor(method, operands, bci, opcode)?;
            let (params, returns) = parse_method_descriptor(&descriptor)?;
            // The arguments sit above the receiver and are consumed in reverse order; the
            // descriptor is what decides how many of them there are and how wide.
            for param in params.iter().rev() {
                frame.pop_ty(param.ty, bci, opcode, false)?;
            }
            if receiver {
                // An `invokespecial <init>` is the call that flips its receiver's initialization
                // state. This slice records the descriptor's shape — an `<init>` returns `void` —
                // and stops on the receiver above instead of claiming the flip, which is 4.2's
                // analysis.
                frame.pop_ty(Ty::Ref, bci, opcode, false)?;
            }
            match returns {
                Some(returns) => frame.push(value_of(returns.ty, returns.name, method), bci),
                None => Ok(()),
            }
        }
        PoolEffect::New => frame.push(Value::Uninitialized { new_site: bci }, bci),
        PoolEffect::NewArray => {
            frame.pop_ty(Ty::Int, bci, opcode, false)?;
            let Some(atype) = operands.atype else {
                return inconsistent(format!(
                    "`newarray` at BCI {bci} names no element type code, which its encoding \
                     requires"
                ));
            };
            if primitive_array_of(atype).is_none() {
                return inconsistent(format!(
                    "`newarray` at BCI {bci} names the element type code {atype}, which is not an \
                     array element type (4..=11)"
                ));
            }
            // The array of a primitive element type is defined by the bootstrap loader, which
            // this request does not declare, so the slot holds a conservative unknown reference
            // instead of a name this request cannot anchor.
            frame.push(Value::Ref(RefType::Unknown), bci)
        }
        PoolEffect::ANewArray => {
            frame.pop_ty(Ty::Int, bci, opcode, false)?;
            let name = pool_class_name(method, operands, bci, opcode)?;
            frame.push(named(array_of(&name), method), bci)
        }
        PoolEffect::MultiANewArray => {
            let name = pool_class_name(method, operands, bci, opcode)?;
            let rank = name.iter().take_while(|byte| **byte == b'[').count();
            let Some(dimensions) = operands.dimensions else {
                return inconsistent(format!(
                    "`multianewarray` at BCI {bci} names no dimension count, which its encoding \
                     requires"
                ));
            };
            if rank == 0 || usize::from(dimensions) > rank {
                return inconsistent(format!(
                    "`multianewarray` at BCI {bci} creates {dimensions} dimension(s) of `{}`, \
                     which is not an array type of at least that rank",
                    String::from_utf8_lossy(&name)
                ));
            }
            // D41: the instruction pops one length per dimension, so its depth change is
            // `1 - dimensions`, and the count that decides it is the reader's own operand fact.
            for _ in 0..dimensions {
                frame.pop_ty(Ty::Int, bci, opcode, false)?;
            }
            frame.push(named(name, method), bci)
        }
        PoolEffect::AThrow => {
            frame.pop_ty(Ty::Ref, bci, opcode, false)?;
            // The frame this instruction ends carries nothing: control leaves the method with the
            // throwable, and what was below it on the stack is discarded rather than moved.
            frame.stack.clear();
            Ok(())
        }
        PoolEffect::CheckCast => {
            frame.pop_ty(Ty::Ref, bci, opcode, false)?;
            let name = pool_class_name(method, operands, bci, opcode)?;
            // The checked class is the type the reference has after the instruction: the cast is
            // what makes the type known, and the name is the class file's own.
            frame.push(named(name, method), bci)
        }
    }
}

/// The indices, into the decoded facts, of the instructions one canonical node runs.
///
/// A node is not a BCI range: the normalization fuses a chain of single transfers into one super
/// block and clones keep the subroutine's own blocks, so a node's span can hold the instructions
/// of a **different** node — a chain jumping over the arm of a diamond would take that arm's
/// instructions with it. What a node runs is therefore its own original blocks, each walked from
/// its own start to the instruction that ends it, and the last one to the node's end.
fn instruction_indices(block: &CanonicalBlock, facts: &MethodCodeFacts) -> Norm<Vec<usize>> {
    let operands = facts.operands();
    let mut indices = Vec::new();
    for (position, start) in block.blocks.iter().enumerate() {
        let limit = block
            .blocks
            .get(position + 1)
            .copied()
            .unwrap_or(block.end_bci);
        let mut index = facts
            .instructions
            .partition_point(|instruction| instruction.bci < *start);
        if facts.instructions.get(index).map(|fact| fact.bci) != Some(*start) {
            return inconsistent(format!(
                "block {:?} names the original block start {start}, which is not an instruction \
                 of this body",
                block.id
            ));
        }
        while let Some(instruction) = facts.instructions.get(index) {
            if instruction.bci >= limit {
                break;
            }
            let Some(operands) = operands.get(index) else {
                return inconsistent(format!(
                    "BCI {} has no operand facts, and the reader produces the two in lockstep",
                    instruction.bci
                ));
            };
            indices.push(index);
            index += 1;
            if TABLE[usize::from(operands.effective_opcode)].ends_block {
                break;
            }
        }
    }
    if indices.is_empty() {
        return inconsistent(format!(
            "block {:?} covers no decoded instruction: none of its original blocks {:?} holds one",
            block.id, block.blocks
        ));
    }
    Ok(indices)
}

/// The exit state of one node: its entry state with every instruction it runs applied in order.
fn transfer_block(
    method: &FrameMethod<'_>,
    block: &CanonicalBlock,
    facts: &MethodCodeFacts,
    entry: Frame,
    budget: &mut Budget,
) -> Norm<Frame> {
    let mut frame = entry;
    let operands = facts.operands();
    for index in instruction_indices(block, facts)? {
        let instruction = &facts.instructions[index];
        let operands = &operands[index];
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let row = TABLE[usize::from(operands.effective_opcode)];
        apply(method, row, instruction.bci, operands, &mut frame)?;
    }
    Ok(frame)
}

/// The frame the method's own entry block is entered with: the parameters in the slots their
/// descriptor gives them, and `this` where the declaration puts it.
///
/// A constructor's local 0 is [`Value::UninitializedThis`] — the token 4.2 flips — and every other
/// instance method's is an initialized reference to the declaring class. The descriptor and the
/// access flags decide both; nothing here guesses from the body.
fn entry_frame(method: &FrameMethod<'_>, facts: &MethodCodeFacts) -> Norm<Frame> {
    let slots = usize::from(facts.max_locals);
    let mut locals = vec![Value::Top; slots];
    let mut next = 0usize;
    if method.access_flags & ACC_STATIC == 0 {
        if slots == 0 {
            return inconsistent(format!(
                "the instance method `{}` declares no local slot for its own receiver",
                String::from_utf8_lossy(method.name)
            ));
        }
        locals[0] = if method.name == b"<init>" {
            Value::UninitializedThis
        } else {
            named(method.owner.to_vec(), method)
        };
        next = 1;
    }
    let (params, _returns) = parse_method_descriptor(method.descriptor)?;
    for param in params {
        let width = param.ty.slots();
        if next + width > slots {
            return inconsistent(format!(
                "the descriptor `{}` needs more local slots than the method declares \
                 (max_locals {slots})",
                String::from_utf8_lossy(method.descriptor)
            ));
        }
        locals[next] = value_of(param.ty, param.name, method);
        if width == 2 {
            locals[next + 1] = Value::Second;
        }
        next += width;
    }
    Ok(Frame {
        locals,
        stack: Vec::new(),
    })
}

/// What one run of the `frame` pass produced.
#[derive(Debug)]
pub(crate) enum FrameOutcome {
    /// The frames of every block the method's entry reaches.
    Frames(Box<FrameTable>),
    /// This build does not state the frames of this body yet: the initialization conversions and
    /// the handler entries are 4.2's. The message names which of the two stopped the run.
    Unsupported { message: String },
    /// The body contradicts itself; the message names the slot and the class file's own operand.
    Inconsistent { message: String },
}

/// The entry state of one canonical block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BlockFrame {
    /// Identity of the block this state belongs to.
    pub(crate) block: CanonicalBlockId,
    /// One entry per local slot.
    pub(crate) locals: Vec<Value>,
    /// The operand stack, bottom first.
    pub(crate) stack: Vec<Value>,
}

/// The published artifact: the entry state of every block the entry reaches.
///
/// The table is derived storage whose only consumers in this build are the later slices — 4.2's
/// initialization analysis and 4.3's SSA — and 5.1 decides what becomes public, so it stays
/// crate-private exactly like the canonical graph it is derived from.
#[allow(
    dead_code,
    reason = "4.2 and 4.3 consume this payload; 5.1 decides its surface"
)]
#[derive(Debug)]
pub(crate) struct FrameTable {
    /// One entry per reached block, in the canonical graph's block order.
    blocks: Vec<BlockFrame>,
    /// Slots of one locals array of this body (`max_locals`).
    locals_slots: usize,
    /// Deepest operand stack any entry state reaches, in slots.
    deepest_stack: usize,
}

#[allow(
    dead_code,
    reason = "4.2 and 4.3 consume this payload; 5.1 decides its surface"
)]
impl FrameTable {
    /// The entry states, in the canonical graph's block order.
    pub(crate) fn blocks(&self) -> &[BlockFrame] {
        &self.blocks
    }

    /// The entry state of one block, or `None` when the entry cannot reach it.
    pub(crate) fn entry(&self, block: &CanonicalBlockId) -> Option<&BlockFrame> {
        self.blocks.iter().find(|entry| &entry.block == block)
    }

    /// Slots of one locals array of this body.
    pub(crate) fn locals_slots(&self) -> usize {
        self.locals_slots
    }

    /// Deepest operand stack any block is entered with.
    pub(crate) fn deepest_stack(&self) -> usize {
        self.deepest_stack
    }
}

/// Builds the frames of one canonical graph.
///
/// The entry state of each block is the **merge of its predecessors' exit states**, computed by a
/// worklist over the canonical transfers until nothing changes. `Frames` covers every block the
/// entry reaches, so a body whose decode stopped early has the frames of its reliable prefix,
/// exactly like the passes before this one. An unproven state and a contradiction are two
/// different outcomes, and neither of them is silently absorbed into the other.
///
/// Charges, in order: one `IrItems` per slot of a block's entry state **before** that state is
/// stored (the locals array and the operand stack it is entered with), one more per merge that
/// changes one, one `AnalysisSteps` per worklist pop and per instruction examined, and one
/// `IrItems` for the published table. The operand stack an instruction builds *inside* a block is
/// not derived storage: it never leaves that block's transfer and the JVM's own slot ceiling
/// bounds it.
pub(crate) fn frames(
    facts: &MethodCodeFacts,
    canonical: &CanonicalCfg,
    method: &FrameMethod<'_>,
    budget: &mut Budget,
) -> Result<FrameOutcome> {
    match run(facts, canonical, method, budget) {
        Ok(table) => Ok(FrameOutcome::Frames(Box::new(table))),
        Err(Problem::Budget(error)) => Err(error),
        Err(Problem::Inconsistent(message)) => Ok(FrameOutcome::Inconsistent { message }),
        Err(Problem::Unproven(message)) => Ok(FrameOutcome::Unsupported { message }),
    }
}

/// The fixpoint itself, as [`frames`] documents it.
fn run(
    facts: &MethodCodeFacts,
    canonical: &CanonicalCfg,
    method: &FrameMethod<'_>,
    budget: &mut Budget,
) -> Norm<FrameTable> {
    let mut index_of: BTreeMap<CanonicalBlockId, usize> = BTreeMap::new();
    for (position, block) in canonical.blocks.iter().enumerate() {
        index_of.insert(block.id.clone(), position);
    }
    let mut successors: Vec<Vec<(usize, CanonicalEdgeKind)>> =
        vec![Vec::new(); canonical.blocks.len()];
    for edge in &canonical.edges {
        let (Some(&from), Some(&to)) = (index_of.get(&edge.from), index_of.get(&edge.to)) else {
            // The canonical post-condition rules this out, so an edge naming a block the graph
            // does not hold is a defect of the graph and not a fact about the method.
            return inconsistent(format!(
                "the canonical edge {edge:?} names a block the graph does not hold"
            ));
        };
        successors[from].push((to, edge.kind));
    }
    let entry_id = CanonicalBlockId {
        bci: 0,
        path: Vec::new(),
    };
    let Some(&entry) = index_of.get(&entry_id) else {
        return inconsistent(format!(
            "the canonical graph holds no entry block {entry_id:?} for a body that starts at BCI 0"
        ));
    };

    checkpoint(Phase::Entry, budget)?;
    let first = entry_frame(method, facts)?;
    charge_frame(budget, &first)?;
    let mut entries: Vec<Option<Frame>> = vec![None; canonical.blocks.len()];
    let mut exits: Vec<Option<Frame>> = vec![None; canonical.blocks.len()];
    entries[entry] = Some(first);
    let mut worklist = VecDeque::from([entry]);
    while let Some(position) = worklist.pop_front() {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        checkpoint(Phase::Transfer, budget)?;
        let block = canonical.blocks[position].clone();
        let entry_state = entries[position]
            .clone()
            .expect("a block is queued only after it has an entry state");
        let exit = transfer_block(method, &block, facts, entry_state, budget)?;
        if exits[position].as_ref() == Some(&exit) {
            // The transfer is a function of the entry state, so an unchanged exit state cannot
            // change any successor's: the work is done and re-enqueueing would repeat it.
            continue;
        }
        exits[position] = Some(exit.clone());
        for (target, kind) in &successors[position] {
            if let CanonicalEdgeKind::Exception { handler_ordinal } = kind {
                return Err(Problem::Unproven(format!(
                    "block {:?} is entered through the exception edge of handler record \
                     {handler_ordinal} from block {:?}: the state a handler is entered with comes \
                     from each throw site on its own, which is 4.2's analysis",
                    canonical.blocks[*target].id, block.id
                )));
            }
            match entries[*target].as_mut() {
                None => {
                    charge_frame(budget, &exit)?;
                    entries[*target] = Some(exit.clone());
                    worklist.push_back(*target);
                }
                Some(current) => {
                    let merged = merge_frame(current, &exit, &canonical.blocks[*target].id)?;
                    if merged != *current {
                        charge_frame(budget, &merged)?;
                        *current = merged;
                        worklist.push_back(*target);
                    }
                }
            }
        }
    }

    checkpoint(Phase::Publish, budget)?;
    // The invariant this pass owes the graph: every block is either reached or one of the nodes
    // the entry cannot reach, which the canonical graph keeps as its own list.
    let unreachable: Vec<&CanonicalBlockId> = canonical.unreachable.iter().collect();
    let missing: Vec<&CanonicalBlockId> = canonical
        .blocks
        .iter()
        .zip(entries.iter())
        .filter(|(block, state)| state.is_none() && !unreachable.contains(&&block.id))
        .map(|(block, _)| &block.id)
        .collect();
    debug_assert!(
        missing.is_empty(),
        "every canonical block is reached or listed as unreachable, found {missing:?}"
    );

    let deepest_stack = entries
        .iter()
        .flatten()
        .map(|frame| frame.stack.len())
        .max()
        .unwrap_or(0);
    let mut blocks = Vec::new();
    for (position, state) in entries.into_iter().enumerate() {
        let Some(frame) = state else {
            continue;
        };
        blocks.push(BlockFrame {
            block: canonical.blocks[position].id.clone(),
            locals: frame.locals,
            stack: frame.stack,
        });
    }
    budget.charge(CountedBudgetDimension::IrItems, 1)?;
    Ok(FrameTable {
        blocks,
        locals_slots: usize::from(facts.max_locals),
        deepest_stack,
    })
}

/// Charges one derived frame state, per slot, before it is stored.
fn charge_frame(budget: &mut Budget, frame: &Frame) -> Norm<()> {
    let slots = frame.locals.len().saturating_add(frame.stack.len());
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(slots).unwrap_or(u64::MAX),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call_context::{CallContextOutcome, call_contexts};
    use crate::cfg::raw_cfg;
    use jarde_reader::budget::{BudgetDimension, Limits, UsageSnapshot};
    use jarde_reader::classfile::{
        BytecodeStop, ExceptionHandlerFact, InstructionFact, LocalOperand, class_facts,
        method_code_facts,
    };
    use jarde_reader::model::{
        ByteSpan, ClassBytesId, Digest, ExecutionReport, JvmBytes, PhysicalClassLocation,
        PhysicalDefinitionId, PhysicalMethodId, PhysicalVariant, SnapshotId, TerminationReason,
    };

    /// Class-file offset the synthetic fixture bodies start at.
    const CODE_OFFSET: u64 = 200;

    /// Two arms of one conditional that reach their join with operand stacks of different depth:
    /// the taken arm has popped its condition, the fall-through arm has pushed a value.
    const DEPTH_CONFLICT_BODY: &[u8] = &[0x03, 0x99, 0x00, 0x04, 0x03, 0xb1];

    /// Two arms that reach their join with one slot each, of two different classes.
    #[rustfmt::skip]
    const CLASS_CONFLICT_BODY: &[u8] = &[
        0x03,                   // iconst_0
        0x99, 0x00, 0x07,       // ifeq +7 -> 8
        0x03,                   // iconst_0
        0xa7, 0x00, 0x07,       // goto +7 -> 12
        0x0b,                   // fconst_0
        0xa7, 0x00, 0x03,       // goto +3 -> 12
        0xb1,                   // return
    ];

    fn limits() -> Limits {
        Limits {
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
            normalization_clones: 1 << 20,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        }
    }

    fn budget() -> Budget {
        Budget::new(limits())
    }

    fn limits_with(change: impl FnOnce(&mut Limits)) -> Limits {
        let mut limits = limits();
        change(&mut limits);
        limits
    }

    /// The identity the origins of the synthetic bodies are anchored to.
    fn method_id() -> PhysicalMethodId {
        PhysicalMethodId {
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
            name: JvmBytes(b"method".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        }
    }

    /// The canonical graph of one decoded body, through the payload 3.4b establishes.
    fn canonical_of(facts: &MethodCodeFacts, major: u16) -> CanonicalCfg {
        let raw = raw_cfg(facts, &mut budget()).expect("the fixture body has a raw graph");
        let contexts = match call_contexts(facts, &raw, major, &mut budget())
            .expect("the call-context walk runs")
        {
            CallContextOutcome::Established(contexts) => contexts,
            other => panic!("the fixture must establish contexts, got {other:?}"),
        };
        match crate::canonical::canonical_cfg(facts, &raw, &contexts, &method_id(), &mut budget())
            .expect("the budget is ample")
        {
            crate::canonical::CanonicalOutcome::Canonical(graph) => *graph,
            crate::canonical::CanonicalOutcome::Fallback { message } => {
                panic!("the fixture must normalize, but stopped: {message}")
            }
        }
    }

    /// One body of the fixtures below: the facts, the class file's own constant pool and the
    /// canonical graph of the body.
    struct Fixture {
        facts: MethodCodeFacts,
        pool: Vec<CpEntryFacts>,
        canonical: CanonicalCfg,
    }

    /// The decoded facts of `Test.method()V` with the caller's body, through the real reader: a
    /// class file with no `StackMapTable`, `LineNumberTable` or local-variable table, whose body
    /// is the one the caller wrote.
    fn fixture_body(code: &[u8], max_locals: u16) -> Fixture {
        let bytes = jarde_reader::classfile::test_class::single_method(52, 64, max_locals, code);
        fixture_of(&bytes)
    }

    /// The same for one class file the caller built itself.
    fn fixture_of(bytes: &[u8]) -> Fixture {
        let mut budget = budget();
        let header = class_facts(bytes, &mut budget).expect("the fixture is a class file");
        let member = header
            .methods
            .first()
            .expect("the fixture declares one method");
        let facts = method_code_facts(bytes, member, &mut budget).expect("the fixture decodes");
        let pool = header.constant_pool.clone();
        let canonical = canonical_of(&facts, 52);
        Fixture {
            facts,
            pool,
            canonical,
        }
    }

    /// One diamond: a condition, a then-arm that jumps to the join, an else-arm that falls into
    /// it, and the BCI the join starts at.
    ///
    /// The shape matters and is the reason this helper exists: the normalization fuses a block
    /// into its successor when that successor is reached by **one** edge alone, so a diamond's
    /// join is *not* fused and really is entered as a block whose state is the merge of both arms.
    /// A body written as `goto` plus a fall-through arm has no join at all — the arm is dead code
    /// and the two paths never meet.
    fn diamond(prologue: &[u8], then_arm: &[u8], else_arm: &[u8]) -> (Vec<u8>, u32) {
        let base = u32::try_from(prologue.len()).expect("a small fixture");
        let then_start = base + 4;
        let else_start = then_start + u32::try_from(then_arm.len()).expect("a small fixture") + 3;
        let join = else_start + u32::try_from(else_arm.len()).expect("a small fixture");
        let offset = |target: u32, at: u32| {
            i16::try_from(
                i32::try_from(target).expect("a small fixture")
                    - i32::try_from(at).expect("a small fixture"),
            )
            .expect("a fixture offset")
            .to_be_bytes()
        };
        let mut code = prologue.to_vec();
        code.push(0x03); // iconst_0, the condition
        code.push(0x99); // ifeq
        code.extend_from_slice(&offset(else_start, base + 1));
        code.extend_from_slice(then_arm);
        code.push(0xa7); // goto the join
        code.extend_from_slice(&offset(
            join,
            then_start + u32::try_from(then_arm.len()).expect("a small fixture"),
        ));
        code.extend_from_slice(else_arm);
        (code, join)
    }

    /// The method view of one fixture body, declared the way the builder declares it: a static
    /// `Test.method` with the descriptor the caller names.
    fn method_of<'a>(
        fixture: &'a Fixture,
        loader: &'a LoaderId,
        descriptor: &'a [u8],
    ) -> FrameMethod<'a> {
        FrameMethod {
            access_flags: ACC_STATIC,
            name: b"method",
            descriptor,
            owner: b"Test",
            pool: &fixture.pool,
            loader,
        }
    }

    /// One run of the pass over a fixture body.
    fn frames_of(fixture: &Fixture) -> FrameOutcome {
        let loader = LoaderId("app".to_string());
        let method = method_of(fixture, &loader, b"()V");
        frames(&fixture.facts, &fixture.canonical, &method, &mut budget())
            .expect("a legal run is answered")
    }

    /// The frames of one body, or the panic that says the body should have been analyzed.
    fn frames_or_panic(outcome: FrameOutcome) -> Box<FrameTable> {
        match outcome {
            FrameOutcome::Frames(table) => table,
            other => panic!("the body must be analyzed, got {other:?}"),
        }
    }

    /// The message of a contradiction, or the panic that says it was something else.
    fn inconsistent(outcome: FrameOutcome) -> String {
        match outcome {
            FrameOutcome::Inconsistent { message } => message,
            other => panic!("the body must contradict itself, got {other:?}"),
        }
    }

    /// The message of a state this build does not prove, or the panic that says it was something
    /// else — a bytecode contradiction in particular, which must never be reported under this
    /// boundary code.
    fn unproven(outcome: FrameOutcome) -> String {
        match outcome {
            FrameOutcome::Unsupported { message } => message,
            other => panic!("the body must stop on the 4.2 boundary, got {other:?}"),
        }
    }

    /// The entry state of one canonical block, by BCI, for a body whose blocks are all top level.
    fn entry_of(table: &FrameTable, bci: u32) -> BlockFrame {
        table
            .entry(&CanonicalBlockId {
                bci,
                path: Vec::new(),
            })
            .unwrap_or_else(|| panic!("the entry reaches the block at BCI {bci}"))
            .clone()
    }

    // -- The dense table -----------------------------------------------------------------

    /// Every opcode value has a row, and the rows the opcode alone decides agree with the raw
    /// pass's own tables: the stack delta and the block-ender classification. The two answer the
    /// same questions from two different places, so a disagreement is one of them being wrong.
    #[test]
    fn the_dense_table_covers_the_opcode_space_and_agrees_with_the_raw_pass() {
        for opcode in 0x00u16..=0xff {
            let opcode = u8::try_from(opcode).expect("a byte");
            let row = TABLE[usize::from(opcode)];
            let decodable = opcode <= 0xc9 && opcode != 0xc4;
            assert_eq!(
                row.stack != Stack::NotAnOpcode,
                decodable,
                "{opcode:#04x} has a row exactly when it is an instruction"
            );
            assert_eq!(
                row.ends_block,
                crate::cfg::ends_block(opcode),
                "{opcode:#04x}: the dense table and the raw pass agree on where a block ends"
            );
        }
        // `0xc4` is the `wide` prefix: the operands' effective opcode is what reaches the table.
        assert_eq!(TABLE[0xc4].stack, Stack::NotAnOpcode);

        let mut checked = 0usize;
        for opcode in 0x00u16..=0xc9 {
            let opcode = u8::try_from(opcode).expect("a byte");
            let Some(delta) = TABLE[usize::from(opcode)].fixed_delta() else {
                continue;
            };
            assert_eq!(
                crate::cfg::fixed_stack_delta(opcode, Some(1)),
                Some(delta),
                "{opcode:#04x}: the dense table and the raw pass agree on the stack delta"
            );
            checked += 1;
        }
        assert!(
            checked >= 150,
            "the cross-check covers the rows whose shape the opcode decides: {checked}"
        );
    }

    /// Every row of the comparison and branch family names the operand classes JVMS 6.5 gives that
    /// opcode, one opcode at a time. The family is not one range of one shape: `0xa5`/`0xa6`
    /// continue the `if` forms in the opcode space but compare **references**, and a range that
    /// swallowed them as `if_icmp` shapes made a legal reference comparison look like a body that
    /// contradicts itself. The cross-check above cannot see that difference — two `int`s and two
    /// references are one and the same slot delta — so this is what holds the classes.
    #[test]
    fn the_comparison_and_branch_family_names_its_operand_classes_per_opcode() {
        /// The pops and pushes JVMS 6.5 gives one opcode of the family.
        fn shape(opcode: u8) -> (&'static [Ty], &'static [Produced]) {
            match opcode {
                0x94 => (POP_LL, PUSH_I),           // lcmp
                0x95 | 0x96 => (POP_FF, PUSH_I),    // fcmpl, fcmpg
                0x97 | 0x98 => (POP_DD, PUSH_I),    // dcmpl, dcmpg
                0x99..=0x9e => (POP_I, PUSH_NONE),  // ifeq..ifle: one `int`, and no push
                0x9f..=0xa4 => (POP_II, PUSH_NONE), // if_icmpeq..if_icmple
                0xa5 | 0xa6 => (POP_RR, PUSH_NONE), // if_acmpeq, if_acmpne
                other => panic!("{other:#04x} is not a comparison or a branch of the family"),
            }
        }

        for opcode in 0x94u8..=0xa6 {
            let (pops, pushes) = shape(opcode);
            assert_eq!(
                TABLE[usize::from(opcode)].stack,
                Stack::Fixed { pops, pushes },
                "{opcode:#04x}: the row pops {pops:?} and pushes {pushes:?}"
            );
        }
    }

    // -- The families whose wrong row has the same depth --------------------------------

    // Each test below names the operand classes JVMS 6.5 gives one opcode, one opcode at a time,
    // and **spells the sequences out** instead of comparing a row with the shorthand it is written
    // with: a row compared with its own shorthand says nothing about that shorthand, and one of
    // the rows below was wrong inside the shorthand. (The empty sequence stays `PUSH_NONE`; there
    // is nothing in it to spell.) The comparison family above is the same kind of test, and
    // `0xa5`/`0xa6` are its member of that shape.

    /// Every row of the shift family names the operand classes JVMS 6.5 gives that opcode, one
    /// opcode at a time. The family is one and the same depth change either way — `ishl` takes two
    /// `int`s and `lshl` an `int` over a `long`, two slots either way — so the cross-check above,
    /// which counts slots, cannot see the classes. A row written in the **prose** order of the
    /// JVMS sentence (the `long` value named first) put the `long` where the shift distance is:
    /// `long >> n`, the everyday output of javac, then looked like a body that contradicts itself,
    /// and the mirror body — a `long` above an `int` — was accepted in its place.
    #[test]
    fn the_shift_family_names_its_operand_classes_per_opcode() {
        /// The pops and pushes JVMS 6.5 gives one opcode of the family.
        fn shape(opcode: u8) -> (&'static [Ty], &'static [Produced]) {
            match opcode {
                0x78 | 0x7a | 0x7c => (&[Ty::Int, Ty::Int], &[Produced::Int]), // ishl, ishr, iushr
                0x79 | 0x7b | 0x7d => (&[Ty::Int, Ty::Long], &[Produced::Long]), // lshl, lshr, lushr
                other => panic!("{other:#04x} is not a shift of the family"),
            }
        }

        for opcode in 0x78u8..=0x7d {
            let (pops, pushes) = shape(opcode);
            assert_eq!(
                TABLE[usize::from(opcode)].stack,
                Stack::Fixed { pops, pushes },
                "{opcode:#04x}: the row pops {pops:?} and pushes {pushes:?}"
            );
        }
    }

    /// Every row of the array-load family names the operand classes JVMS 6.5 gives that opcode:
    /// the index on top of the array reference, and one element class per opcode. `iaload` and
    /// `aaload` are the same depth change and differ all the way down — the reference pops the
    /// reference and pushes a reference, the integer form pushes an `int`.
    #[test]
    fn the_array_load_family_names_its_operand_classes_per_opcode() {
        /// The pops and pushes JVMS 6.5 gives one opcode of the family.
        fn shape(opcode: u8) -> (&'static [Ty], &'static [Produced]) {
            match opcode {
                0x2e => (&[Ty::Int, Ty::Ref], &[Produced::Int]), // iaload
                0x2f => (&[Ty::Int, Ty::Ref], &[Produced::Long]), // laload
                0x30 => (&[Ty::Int, Ty::Ref], &[Produced::Float]), // faload
                0x31 => (&[Ty::Int, Ty::Ref], &[Produced::Double]), // daload
                0x32 => (&[Ty::Int, Ty::Ref], &[Produced::Ref]), // aaload
                0x33..=0x35 => (&[Ty::Int, Ty::Ref], &[Produced::Int]), // baload, caload, saload
                other => panic!("{other:#04x} is not an array load of the family"),
            }
        }

        for opcode in 0x2eu8..=0x35 {
            let (pops, pushes) = shape(opcode);
            assert_eq!(
                TABLE[usize::from(opcode)].stack,
                Stack::Fixed { pops, pushes },
                "{opcode:#04x}: the row pops {pops:?} and pushes {pushes:?}"
            );
        }
    }

    /// Every row of the array-store family names the operand classes JVMS 6.5 gives that opcode,
    /// one opcode at a time: the value on top, the index below it, the array reference below
    /// both. `iastore` and `aastore` are one depth change — three slots either way — and only the
    /// top class tells them apart.
    #[test]
    fn the_array_store_family_names_its_operand_classes_per_opcode() {
        /// The pops JVMS 6.5 gives one opcode of the family, none of which pushes.
        fn shape(opcode: u8) -> &'static [Ty] {
            match opcode {
                0x4f | 0x54..=0x56 => &[Ty::Int, Ty::Int, Ty::Ref], // iastore, bastore, castore
                0x50 => &[Ty::Long, Ty::Int, Ty::Ref],              // lastore
                0x51 => &[Ty::Float, Ty::Int, Ty::Ref],             // fastore
                0x52 => &[Ty::Double, Ty::Int, Ty::Ref],            // dastore
                0x53 => &[Ty::Ref, Ty::Int, Ty::Ref],               // aastore
                other => panic!("{other:#04x} is not an array store of the family"),
            }
        }

        for opcode in 0x4fu8..=0x56 {
            let pops = shape(opcode);
            assert_eq!(
                TABLE[usize::from(opcode)].stack,
                Stack::Fixed {
                    pops,
                    pushes: PUSH_NONE
                },
                "{opcode:#04x}: the row pops {pops:?} and pushes nothing"
            );
        }
    }

    /// Every row of the conversion family names the classes JVMS 6.5 gives that opcode, one
    /// opcode at a time. The family is where a wrong row is hardest to notice: every conversion of
    /// one and the same width `w` changes the depth by the same amount, so `i2l` and `i2d` — an
    /// `int` to a `long` and an `int` to a `double` — are one delta, and only the pushed class
    /// separates them.
    #[test]
    fn the_conversion_family_names_its_operand_classes_per_opcode() {
        /// The pops and pushes JVMS 6.5 gives one opcode of the family.
        fn shape(opcode: u8) -> (&'static [Ty], &'static [Produced]) {
            match opcode {
                0x85 => (&[Ty::Int], &[Produced::Long]),       // i2l
                0x86 => (&[Ty::Int], &[Produced::Float]),      // i2f
                0x87 => (&[Ty::Int], &[Produced::Double]),     // i2d
                0x88 => (&[Ty::Long], &[Produced::Int]),       // l2i
                0x89 => (&[Ty::Long], &[Produced::Float]),     // l2f
                0x8a => (&[Ty::Long], &[Produced::Double]),    // l2d
                0x8b => (&[Ty::Float], &[Produced::Int]),      // f2i
                0x8c => (&[Ty::Float], &[Produced::Long]),     // f2l
                0x8d => (&[Ty::Float], &[Produced::Double]),   // f2d
                0x8e => (&[Ty::Double], &[Produced::Int]),     // d2i
                0x8f => (&[Ty::Double], &[Produced::Long]),    // d2l
                0x90 => (&[Ty::Double], &[Produced::Float]),   // d2f
                0x91..=0x93 => (&[Ty::Int], &[Produced::Int]), // i2b, i2c, i2s
                other => panic!("{other:#04x} is not a conversion of the family"),
            }
        }

        for opcode in 0x85u8..=0x93 {
            let (pops, pushes) = shape(opcode);
            assert_eq!(
                TABLE[usize::from(opcode)].stack,
                Stack::Fixed { pops, pushes },
                "{opcode:#04x}: the row pops {pops:?} and pushes {pushes:?}"
            );
        }
    }

    /// Every row of the return family names the one class it hands back, per opcode, and none of
    /// them pushes. All six leave one slot for the caller to read — `ireturn` one `int`,
    /// `lreturn` one `long` — and `return` none at all; the classes are what a caller's frame
    /// would be entered with, so a wrong row here moves the mistake into the callee's state.
    #[test]
    fn the_return_family_names_the_class_it_returns_per_opcode() {
        /// The pops JVMS 6.5 gives one opcode of the family, none of which pushes.
        fn shape(opcode: u8) -> &'static [Ty] {
            match opcode {
                0xac => &[Ty::Int],    // ireturn
                0xad => &[Ty::Long],   // lreturn
                0xae => &[Ty::Float],  // freturn
                0xaf => &[Ty::Double], // dreturn
                0xb0 => &[Ty::Ref],    // areturn
                0xb1 => &[],           // return
                other => panic!("{other:#04x} is not a return of the family"),
            }
        }

        for opcode in 0xacu8..=0xb1 {
            let pops = shape(opcode);
            assert_eq!(
                TABLE[usize::from(opcode)].stack,
                Stack::Fixed {
                    pops,
                    pushes: PUSH_NONE
                },
                "{opcode:#04x}: the row pops {pops:?} and pushes nothing"
            );
        }
    }

    /// The two singletons the opcode alone decides take a reference and nothing else:
    /// `arraylength` pops the array and pushes its length, `instanceof` pops the reference and
    /// pushes the `int` answer — an `int` and not a reference, which is why the answer is a
    /// boolean in the source language and a one-slot computation type here.
    ///
    /// `checkcast` is the third singleton of the group and the one row whose shape is a
    /// **class-file fact** ([`PoolEffect::CheckCast`]): it pops a reference and pushes the class
    /// file's own class, which the row cannot spell. Its operand classes are therefore pinned by
    /// the real bodies below and not by the table alone.
    #[test]
    fn the_reference_singletons_name_their_operand_classes_per_opcode() {
        /// The pops and pushes JVMS 6.5 gives one opcode of the family the opcode decides.
        fn shape(opcode: u8) -> (&'static [Ty], &'static [Produced]) {
            match opcode {
                0xbe => (&[Ty::Ref], &[Produced::Int]), // arraylength: array in, length out
                0xc1 => (&[Ty::Ref], &[Produced::Int]), // instanceof: reference in, answer out
                other => panic!("{other:#04x} is not a fixed-shape singleton of the family"),
            }
        }

        for opcode in [0xbeu8, 0xc1] {
            let (pops, pushes) = shape(opcode);
            assert_eq!(
                TABLE[usize::from(opcode)].stack,
                Stack::Fixed { pops, pushes },
                "{opcode:#04x}: the row pops {pops:?} and pushes {pushes:?}"
            );
        }
        assert_eq!(
            TABLE[0xc0].stack,
            Stack::Constant(PoolEffect::CheckCast),
            "0xc0: the row takes the class it pushes from the class file"
        );

        // `checkcast` leaves a reference: `arraylength` refuses anything else, so a body that
        // reaches `return` through it proves the cast's push. `aconst_null; checkcast [[I;
        // arraylength; pop; return` — constant-pool slot 9 is the class `[[I`.
        let fixture = fixture_body(&[0x01, 0xc0, 0x00, 0x09, 0xbe, 0x57, 0xb1], 0);
        frames_or_panic(frames_of(&fixture));

        // `instanceof` leaves an `int`: `iadd` wants two `int`s and refuses anything else, so
        // `aconst_null; instanceof [[I; iconst_0; iadd; pop; return` proves the answer's class.
        let fixture = fixture_body(&[0x01, 0xc1, 0x00, 0x09, 0x03, 0x60, 0x57, 0xb1], 0);
        frames_or_panic(frames_of(&fixture));

        // And the other direction: what these instructions pop is a reference, and an `int` on the
        // stack is the refusal that names the opcode.
        let fixture = fixture_body(&[0x03, 0xc0, 0x00, 0x09, 0x57, 0xb1], 0);
        let message = inconsistent(frames_of(&fixture));
        assert!(
            message.contains("0xc0") && message.contains("Ref"),
            "the cast's refusal names the opcode and the class it wants: {message}"
        );
        let fixture = fixture_body(&[0x03, 0xc1, 0x00, 0x09, 0x57, 0xb1], 0);
        let message = inconsistent(frames_of(&fixture));
        assert!(
            message.contains("0xc1") && message.contains("Ref"),
            "the test's refusal names the opcode and the class it wants: {message}"
        );
    }

    /// Every row of the `dup`/`pop`/`swap` family names the form JVMS 6.5 gives that opcode. The
    /// form decides which category pattern the instruction may see, and the wrong form is not
    /// visible in the depth change: `0x5d dup2_x1` and `0x5e dup2_x2` are both +2 slots, so a row
    /// that named the other one would pass the cross-check above and then refuse — or accept — the
    /// wrong operands for every body that carries it.
    #[test]
    fn the_form_family_names_its_form_per_opcode() {
        /// The form JVMS 6.5 defines one opcode of the family by.
        fn form(opcode: u8) -> Form {
            match opcode {
                0x57 => Form::Pop,
                0x58 => Form::Pop2,
                0x59 => Form::Dup,
                0x5a => Form::DupX1,
                0x5b => Form::DupX2,
                0x5c => Form::Dup2,
                0x5d => Form::Dup2X1,
                0x5e => Form::Dup2X2,
                0x5f => Form::Swap,
                other => panic!("{other:#04x} is not a form of the family"),
            }
        }

        for opcode in 0x57u8..=0x5f {
            let expected = form(opcode);
            assert_eq!(
                TABLE[usize::from(opcode)].stack,
                Stack::Form(expected),
                "{opcode:#04x}: the row is the form {expected:?}"
            );
        }
    }

    /// Every row of the local-access family names the class of the value it moves, per opcode:
    /// the five `load` groups, the five `store` groups, and the shorthand forms that imply their
    /// index. `iload` and `aload` are one depth change and differ all the way down — what enters
    /// the stack is the local's own value, which is why the row carries an effect on the local and
    /// no stack sequence at all.
    #[test]
    fn the_local_access_family_names_the_class_of_its_local_per_opcode() {
        /// The local effect JVMS 6.5 gives one opcode of the family.
        fn local(opcode: u8) -> Local {
            match opcode {
                0x15 | 0x1a..=0x1d => Local::Load(Ty::Int), // iload, iload_0..3
                0x16 | 0x1e..=0x21 => Local::Load(Ty::Long), // lload, lload_0..3
                0x17 | 0x22..=0x25 => Local::Load(Ty::Float), // fload, fload_0..3
                0x18 | 0x26..=0x29 => Local::Load(Ty::Double), // dload, dload_0..3
                0x19 | 0x2a..=0x2d => Local::Load(Ty::Ref), // aload, aload_0..3
                0x36 | 0x3b..=0x3e => Local::Store(Ty::Int), // istore, istore_0..3
                0x37 | 0x3f..=0x42 => Local::Store(Ty::Long), // lstore, lstore_0..3
                0x38 | 0x43..=0x46 => Local::Store(Ty::Float), // fstore, fstore_0..3
                0x39 | 0x47..=0x4a => Local::Store(Ty::Double), // dstore, dstore_0..3
                0x3a | 0x4b..=0x4e => Local::Store(Ty::Ref), // astore, astore_0..3
                other => panic!("{other:#04x} is not a local access of the family"),
            }
        }

        let rows = (0x15u8..=0x19)
            .chain(0x1a..=0x2d)
            .chain(0x36..=0x3a)
            .chain(0x3b..=0x4e);
        let mut checked = 0usize;
        for opcode in rows {
            let expected = local(opcode);
            let row = TABLE[usize::from(opcode)];
            assert_eq!(
                row.local, expected,
                "{opcode:#04x}: the row moves the local's own {expected:?}"
            );
            assert_eq!(
                row.stack,
                Stack::Fixed {
                    pops: &[],
                    pushes: &[]
                },
                "{opcode:#04x}: the move is the local's, not a stack sequence of the row"
            );
            checked += 1;
        }
        assert_eq!(
            checked, 50,
            "the family covers every local access of the opcode space"
        );
    }

    // -- A long shift, over a real body --------------------------------------------------

    /// `lshr` takes a `long` value with an **`int`** shift distance on top of it (JVMS 6.5), and
    /// the table's pop sequences are written top-first. A row that named the distance second read
    /// `long >> n` — the body javac emits for every shift of a `long` — as a contradiction of the
    /// bytes, and accepted the mirror body, a `long` above an `int`, in its place.
    #[test]
    fn a_long_shift_takes_its_distance_on_top_of_the_value() {
        // lconst_0; iconst_2; lshr; pop2; return: the body javac emits for `long >> 2`, which the
        // pass must derive frames for and not read as a contradiction of its own bytes.
        let fixture = fixture_body(&[0x09, 0x05, 0x7b, 0x58, 0xb1], 0);
        frames_or_panic(frames_of(&fixture));

        // iconst_2; lconst_0; lshr; pop2; return: the `long` above the `int`, which is what the
        // wrong row accepted, and the refusal names the opcode and the class it wants there.
        let fixture = fixture_body(&[0x05, 0x09, 0x7b, 0x58, 0xb1], 0);
        let message = inconsistent(frames_of(&fixture));
        assert!(
            message.contains("0x7b") && message.contains("Int") && message.contains("Long"),
            "the refusal names the opcode and the class it wants: {message}"
        );
    }

    // -- The two comparison kinds, over real bodies --------------------------------------

    /// A reference comparison consumes both references it compares, and an integer comparison
    /// still consumes two `int`s. Both halves are asserted over real bodies and their outcomes,
    /// not over the table: reading `if_acmpeq` as an `if_icmp` shape is what made this pass report
    /// `ir_frame_inconsistent` — a contradiction of the *bytes* — for a method every verifier
    /// accepts.
    #[test]
    fn a_reference_comparison_pops_references_and_an_integer_one_does_not() {
        // 0: aconst_null; 1: aconst_null; 2: if_acmpeq +3 -> 5; 5: return; 6: return. Both edges
        // of the comparison reach BCI 5, which the normalization therefore fuses into the block
        // that compares, so the whole body is entered empty or not at all.
        let fixture = fixture_body(&[0x01, 0x01, 0xa5, 0x00, 0x03, 0xb1, 0xb1], 0);
        let table = frames_or_panic(frames_of(&fixture));
        assert!(
            table.blocks().iter().all(|block| block.stack.is_empty()),
            "no block of this body is entered with anything on its operand stack"
        );

        // The same comparison with the branch really skipping an arm: `+6` from BCI 2 reaches BCI
        // 8, the fall-through arm enters at 5 and joins it there, and neither successor is fused
        // away — so each entry state below is the comparison's own exit state.
        let fixture = fixture_body(&[0x01, 0x01, 0xa5, 0x00, 0x06, 0xa7, 0x00, 0x03, 0xb1], 0);
        let table = frames_or_panic(frames_of(&fixture));
        assert!(
            entry_of(&table, 5).stack.is_empty() && entry_of(&table, 8).stack.is_empty(),
            "each successor of the comparison is entered with what the comparison left behind"
        );

        // The other direction, so the repair cannot have gone too far: `if_icmpeq` wants two
        // `int`s, two `null`s are references, and the refusal is the body's own — it names the
        // opcode and the class it wants.
        let fixture = fixture_body(&[0x01, 0x01, 0x9f, 0x00, 0x03, 0xb1, 0xb1], 0);
        let message = inconsistent(frames_of(&fixture));
        assert!(
            message.contains("0x9f") && message.contains("Int"),
            "the refusal names the opcode and the class it wants: {message}"
        );
    }

    // -- The 4.1 acceptance core ---------------------------------------------------------

    /// Two paths that write the same **dead** local with an `int` and a reference still analyze:
    /// the merge of disagreeing locals is not a refusal, and the design's rule is that only a read
    /// of the merged slot fails.
    #[test]
    fn two_disagreeing_writes_to_one_dead_local_merge_and_still_analyze() {
        // then: local 1 <- int; else: local 1 <- null; both enter the join.
        let (mut code, join) = diamond(&[], &[0x03, 0x3c], &[0x01, 0x4c]);
        code.push(0xb1); // return
        let fixture = fixture_body(&code, 4);
        let table = frames_or_panic(frames_of(&fixture));
        let state = entry_of(&table, join);
        assert_eq!(
            state.locals[1],
            Value::Top,
            "a local two paths disagree about is unreadable, not a refusal"
        );
        assert!(
            state.stack.is_empty(),
            "both arms reach the join with an empty operand stack"
        );
        assert_eq!(table.blocks().len(), 4, "four blocks are entered");
    }

    /// The other half of the same rule: the merge is not the failure, the **read** is.
    #[test]
    fn reading_the_merged_local_is_what_fails() {
        let (mut code, join) = diamond(&[], &[0x03, 0x3c], &[0x01, 0x4c]);
        code.extend_from_slice(&[0x1b, 0x57, 0xb1]); // iload_1; pop; return
        assert!(join > 0, "the join is inside the body");
        let fixture = fixture_body(&code, 4);
        let message = inconsistent(frames_of(&fixture));
        assert!(
            message.contains("local 1") && message.contains("no readable value"),
            "the refusal names the slot it read: {message}"
        );
    }

    /// The operand stack is the half that must agree: a depth mismatch between two paths into one
    /// block is a contradiction of the body.
    #[test]
    fn two_paths_into_one_block_must_agree_on_the_stack_depth() {
        let fixture = fixture_body(DEPTH_CONFLICT_BODY, 4);
        let message = inconsistent(frames_of(&fixture));
        assert!(
            message.contains("operand stacks") && message.contains("slots"),
            "the refusal names the two depths: {message}"
        );
    }

    /// Same depth, two different slot classes: still a contradiction, and not a `Top`.
    #[test]
    fn two_paths_into_one_block_must_agree_on_the_slot_classes() {
        let fixture = fixture_body(CLASS_CONFLICT_BODY, 4);
        let message = inconsistent(frames_of(&fixture));
        assert!(
            message.contains("Int") && message.contains("Float"),
            "the refusal names the two classes: {message}"
        );
    }

    // -- Category-2 binding --------------------------------------------------------------

    /// A `long` occupies two slots, and the state says which slot holds what.
    #[test]
    fn a_category_two_value_marks_its_second_slot() {
        // local 1 <- long (slots 1 and 2), then two arms that touch no local at all.
        let (mut code, join) = diamond(&[0x09, 0x40], &[0x03, 0x57], &[0x03, 0x57]);
        code.push(0xb1); // return
        let fixture = fixture_body(&code, 4);
        let table = frames_or_panic(frames_of(&fixture));
        let state = entry_of(&table, join);
        assert_eq!(state.locals[1], Value::Long);
        assert_eq!(
            state.locals[2],
            Value::Second,
            "the upper slot is the value's, not one of its own"
        );
    }

    /// Covering the **upper** slot of a pair drops the value below it: the two slots were one
    /// value, and no stale lower half may survive.
    #[test]
    fn covering_the_upper_slot_of_a_pair_invalidates_the_lower_one() {
        // local 1 <- long; the then-arm stores an int into slot 2, the upper slot of the pair.
        let (mut code, join) = diamond(&[0x09, 0x40], &[0x03, 0x3d], &[0x03, 0x57]);
        code.push(0xb1); // return
        let fixture = fixture_body(&code, 4);
        let table = frames_or_panic(frames_of(&fixture));
        assert_eq!(
            entry_of(&table, join).locals[1],
            Value::Top,
            "the value that was held in slots 1 and 2 no longer exists"
        );
    }

    /// Covering the **lower** slot drops the upper one, and the read of that upper slot is
    /// refused — with the mirror case of covering the upper slot and reading the lower one.
    #[test]
    fn covering_either_slot_of_a_pair_invalidates_the_other() {
        // 0  lconst_0     local 1 <- long (slots 1 and 2)
        // 1  lstore_1
        // 2  iconst_0
        // 3  istore_2     the upper slot of the pair holds an int
        // 4  lload_1      reads the lower slot, which no longer belongs to a value
        // 5  pop2
        // 6  return
        let fixture = fixture_body(&[0x09, 0x40, 0x03, 0x3d, 0x1f, 0x58, 0xb1], 4);
        let message = inconsistent(frames_of(&fixture));
        assert!(
            message.contains("local 1") && message.contains("no readable value"),
            "the refusal names the lower slot: {message}"
        );

        // The mirror: the lower slot is overwritten, and the upper one is read.
        let fixture = fixture_body(&[0x09, 0x40, 0x03, 0x3c, 0x1c, 0x57, 0xb1], 4);
        let message = inconsistent(frames_of(&fixture));
        assert!(
            message.contains("local 2") && message.contains("no readable value"),
            "the refusal names the upper slot: {message}"
        );
    }

    // -- The `dup*`/`pop*`/`swap` forms ---------------------------------------------------

    /// Each form is paired by category: the sequences that fit the value they see complete, and
    /// the ones that do not are refused.
    #[test]
    fn the_dup_family_pairs_its_categories() {
        // `dup2` of one category-2 value, then two `pop2`: legal, and the stack is emptied twice.
        let fixture = fixture_body(&[0x09, 0x5c, 0x58, 0x58, 0xb1], 4);
        assert!(matches!(frames_of(&fixture), FrameOutcome::Frames(_)));

        // `dup2` of two category-1 values, and four `pop`s: also legal.
        let fixture = fixture_body(&[0x03, 0x03, 0x5c, 0x57, 0x57, 0x57, 0x57, 0xb1], 4);
        assert!(matches!(frames_of(&fixture), FrameOutcome::Frames(_)));

        // `pop2` of one category-2 value.
        let fixture = fixture_body(&[0x09, 0x58, 0xb1], 4);
        assert!(matches!(frames_of(&fixture), FrameOutcome::Frames(_)));

        // `dup2` of a single category-1 value is not a form of the instruction: the second
        // category-1 value it needs is not there.
        let fixture = fixture_body(&[0x03, 0x5c, 0xb1], 4);
        let message = inconsistent(frames_of(&fixture));
        assert!(
            message.contains("0x5c"),
            "the refusal names the instruction: {message}"
        );

        // `pop2` of a single category-1 value is not one either.
        let fixture = fixture_body(&[0x03, 0x58, 0xb1], 4);
        assert!(inconsistent(frames_of(&fixture)).contains("0x58"));

        // `swap` needs two category-1 values, and a category-2 one is not one of them.
        let fixture = fixture_body(&[0x09, 0x5f, 0xb1], 4);
        assert!(inconsistent(frames_of(&fixture)).contains("0x5f"));
    }

    // -- Descriptor-driven shapes ---------------------------------------------------------

    /// One constant-pool entry naming one method, as the synthetic fixtures write theirs.
    fn method_ref(index: u16, descriptor: &[u8]) -> CpEntryFacts {
        CpEntryFacts {
            index,
            span: ByteSpan::new(CODE_OFFSET, 0),
            kind: CpEntryKind::MethodRef {
                class_index: 0,
                name_and_type_index: 0,
                owner: JvmBytes(b"Test".to_vec()),
                name: JvmBytes(b"m".to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
        }
    }

    /// One constant-pool entry naming one interface method, as the synthetic fixtures write
    /// theirs.
    fn interface_method_ref(index: u16, descriptor: &[u8]) -> CpEntryFacts {
        CpEntryFacts {
            index,
            span: ByteSpan::new(CODE_OFFSET, 0),
            kind: CpEntryKind::InterfaceMethodRef {
                class_index: 0,
                name_and_type_index: 0,
                owner: JvmBytes(b"Test".to_vec()),
                name: JvmBytes(b"m".to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
        }
    }

    /// One constant-pool entry naming one dynamic call site, as the synthetic fixtures write
    /// theirs.
    fn dynamic_ref(index: u16, descriptor: &[u8]) -> CpEntryFacts {
        CpEntryFacts {
            index,
            span: ByteSpan::new(CODE_OFFSET, 0),
            kind: CpEntryKind::InvokeDynamic {
                bootstrap_method_attr_index: 0,
                name_and_type_index: 0,
                name: JvmBytes(b"m".to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
        }
    }

    /// A synthetic body: hand-written facts, the constant pool the caller names and the canonical
    /// graph the real normalization builds over them.
    struct Synthetic {
        facts: MethodCodeFacts,
        pool: Vec<CpEntryFacts>,
        canonical: CanonicalCfg,
    }

    fn operands(opcode: u8) -> InstructionOperands {
        InstructionOperands {
            effective_opcode: opcode,
            ..InstructionOperands::default()
        }
    }

    fn instruction(
        bci: u32,
        opcode: u8,
        width: u32,
        operands: InstructionOperands,
    ) -> (InstructionFact, InstructionOperands) {
        let start = CODE_OFFSET + u64::from(bci);
        (
            InstructionFact {
                bci,
                opcode,
                width,
                span: ByteSpan::new(start, u64::from(width)),
                operands_span: ByteSpan::new(start + 1, u64::from(width - 1)),
                constant_pool_index: operands.constant_pool_index,
            },
            operands,
        )
    }

    fn plain(bci: u32, opcode: u8) -> (InstructionFact, InstructionOperands) {
        instruction(bci, opcode, 1, operands(opcode))
    }

    /// One form whose local is implicit in the opcode, which the reader records as a fact.
    fn local(bci: u32, opcode: u8, index: u16) -> (InstructionFact, InstructionOperands) {
        instruction(
            bci,
            opcode,
            1,
            InstructionOperands {
                local: Some(LocalOperand { index, wide: false }),
                ..operands(opcode)
            },
        )
    }

    fn call(bci: u32, opcode: u8, index: u16) -> (InstructionFact, InstructionOperands) {
        instruction(
            bci,
            opcode,
            3,
            InstructionOperands {
                constant_pool_index: Some(index),
                ..operands(opcode)
            },
        )
    }

    fn branch(bci: u32, opcode: u8, offset: i32) -> (InstructionFact, InstructionOperands) {
        instruction(
            bci,
            opcode,
            3,
            InstructionOperands {
                branch_offset: Some(offset),
                ..operands(opcode)
            },
        )
    }

    fn synthetic_body(
        code: Vec<(InstructionFact, InstructionOperands)>,
        pool: Vec<CpEntryFacts>,
        max_locals: u16,
        code_length: u32,
    ) -> Synthetic {
        let facts = MethodCodeFacts::from_parts(
            64,
            max_locals,
            ByteSpan::new(CODE_OFFSET, u64::from(code_length)),
            code,
            Vec::new(),
            0,
            ExecutionReport::Complete {
                usage: UsageSnapshot::default(),
            },
            None,
        );
        let canonical = canonical_of(&facts, 52);
        Synthetic {
            facts,
            pool,
            canonical,
        }
    }

    fn synthetic_frames(synthetic: &Synthetic, descriptor: &[u8]) -> FrameOutcome {
        let loader = LoaderId("app".to_string());
        let method = FrameMethod {
            access_flags: ACC_STATIC,
            name: b"method",
            descriptor,
            owner: b"Test",
            pool: &synthetic.pool,
            loader: &loader,
        };
        frames(
            &synthetic.facts,
            &synthetic.canonical,
            &method,
            &mut budget(),
        )
        .expect("a legal run is answered")
    }

    /// An invocation takes its argument count, its widths and its return value from the
    /// descriptor: `(J)V` consumes a two-slot argument and returns nothing, and `(DD)D` consumes
    /// two of them and pushes a two-slot value.
    #[test]
    fn an_invocation_takes_its_shape_from_the_descriptor() {
        // `invokeinterface #15` is `java/lang/Runnable.run:(J)V`: it consumes the object reference
        // and the two slots of the long, and pushes nothing. The arm that does not call leaves the
        // same stack behind by popping both, so the join is entered empty exactly when the call
        // consumed what its descriptor says.
        let (mut code, join) = diamond(
            &[0x01, 0x09],                   // aconst_null (the receiver), lconst_0 (the argument)
            &[0xb9, 0x00, 0x0f, 0x03, 0x00], // invokeinterface #15, count 3
            &[0x58, 0x57],                   // pop2, pop
        );
        code.push(0xb1); // return
        let fixture = fixture_body(&code, 4);
        let table = frames_or_panic(frames_of(&fixture));
        assert!(
            entry_of(&table, join).stack.is_empty(),
            "the receiver and the two slots of the long are what the descriptor consumes"
        );

        // A static call to `(DD)D`: no receiver, two two-slot arguments and a two-slot return
        // value — which is what the join is entered with.
        let synthetic = synthetic_body(
            vec![
                plain(0, 0x0e),      // dconst_0
                plain(1, 0x0f),      // dconst_1
                call(2, 0xb8, 1),    // invokestatic #1 ((DD)D)
                plain(5, 0x03),      // iconst_0 (the condition)
                branch(6, 0x99, 8),  // ifeq +8 -> 14
                plain(9, 0x03),      // iconst_0
                plain(10, 0x57),     // pop
                branch(11, 0xa7, 5), // goto +5 -> 16
                plain(14, 0x03),     // iconst_0
                plain(15, 0x57),     // pop
                plain(16, 0xb1),     // return
            ],
            vec![method_ref(1, b"(DD)D")],
            1,
            17,
        );
        let table = frames_or_panic(synthetic_frames(&synthetic, b"()V"));
        assert_eq!(
            entry_of(&table, 16).stack,
            vec![Value::Double, Value::Second],
            "the descriptor's return type decides what the invocation pushes"
        );
    }

    /// An invocation's opcode and the kind of constant-pool entry it names must be the pair JVMS
    /// 6.5 and SE 8 define. The pass already refuses a `getfield` over a `Methodref`; the method
    /// side had no such pairing, so `invokevirtual` over an `InvokeDynamic` had a shape — receiver
    /// included — read out of an entry the instruction may not name instead of being refused as
    /// the illegal input it is.
    #[test]
    fn an_invocation_names_the_constant_pool_kind_its_opcode_takes() {
        /// One call of `opcode` naming the entry at index 1, at the width its encoding has:
        /// `invokeinterface` also carries its argument count and a zero byte.
        fn call_to(bci: u32, opcode: u8, index: u16) -> (InstructionFact, InstructionOperands) {
            let width = if opcode == 0xb9 { 5 } else { 3 };
            instruction(
                bci,
                opcode,
                width,
                InstructionOperands {
                    constant_pool_index: Some(index),
                    ..operands(opcode)
                },
            )
        }

        /// The body `[aconst_null;] <call #1>; return`, whose `()V` call leaves nothing behind.
        fn body_for(opcode: u8, receiver: bool, entry: CpEntryFacts) -> Synthetic {
            let width = if opcode == 0xb9 { 5 } else { 3 };
            let mut code = Vec::new();
            let mut bci = 0;
            if receiver {
                code.push(plain(0, 0x01)); // aconst_null: the receiver
                bci = 1;
            }
            code.push(call_to(bci, opcode, 1));
            code.push(plain(bci + width, 0xb1)); // return
            synthetic_body(code, vec![entry], 0, bci + width + 1)
        }

        // The three pairings the opcode does not take, each refused as the illegal input it is and
        // with the kind the opcode may not name. `0xba` over a `Methodref`:
        let message = inconsistent(synthetic_frames(
            &body_for(0xba, false, method_ref(1, b"()V")),
            b"()V",
        ));
        assert!(
            message.contains("0xba") && message.contains("Methodref"),
            "the refusal names the opcode and the kind it cannot take: {message}"
        );

        // `0xb9` over a `Methodref`:
        let message = inconsistent(synthetic_frames(
            &body_for(0xb9, true, method_ref(1, b"()V")),
            b"()V",
        ));
        assert!(
            message.contains("0xb9") && message.contains("Methodref"),
            "the refusal names the opcode and the kind it cannot take: {message}"
        );

        // `0xb6` over an `InvokeDynamic`, the pairing whose descriptor used to become the shape of
        // a virtual call:
        let message = inconsistent(synthetic_frames(
            &body_for(0xb6, true, dynamic_ref(1, b"()V")),
            b"()V",
        ));
        assert!(
            message.contains("0xb6") && message.contains("InvokeDynamic"),
            "the refusal names the opcode and the kind it cannot take: {message}"
        );

        // The same pairing over a **real** class file and not a synthetic one: constant-pool slot
        // 15 of the fixture builder is the `InterfaceMethodref` `Runnable.run:(J)V`, and
        // `invokevirtual` may not name it — not a pairing a source-language call can produce, only
        // an illegal class file.
        let fixture = fixture_body(&[0x01, 0xb6, 0x00, 0x0f, 0xb1], 0);
        let message = inconsistent(frames_of(&fixture));
        assert!(
            message.contains("0xb6") && message.contains("InterfaceMethodref"),
            "the refusal names the opcode and the kind it cannot take: {message}"
        );

        // The pairings the opcodes do take, so the pairing cannot have been drawn too narrowly.
        // Since SE 8 `invokespecial` reaches an interface's default and private methods and
        // `invokestatic` its static ones, so both may name an **InterfaceMethodref** — the pairing
        // a blanket "a Methodref only" rule would have refused — and `invokeinterface` names an
        // interface method, `invokedynamic` a call site.
        for (opcode, receiver, entry) in [
            (0xb6u8, true, method_ref(1, b"()V")),
            (0xb7, true, interface_method_ref(1, b"()V")),
            (0xb8, false, interface_method_ref(1, b"()V")),
            (0xb9, true, interface_method_ref(1, b"()V")),
            (0xba, false, dynamic_ref(1, b"()V")),
        ] {
            let outcome = synthetic_frames(&body_for(opcode, receiver, entry), b"()V");
            assert!(
                matches!(outcome, FrameOutcome::Frames(_)),
                "{opcode:#04x} over the entry kind it takes must be analyzed, got {outcome:?}"
            );
        }
    }

    /// The entry frame is the descriptor's own: `this` where the declaration puts it, and one
    /// slot class per parameter, a category-2 one taking two slots.
    #[test]
    fn the_entry_frame_follows_the_declaration() {
        let synthetic = synthetic_body(vec![plain(0, 0xb1)], vec![], 6, 1);
        let loader = LoaderId("app".to_string());
        let instance = FrameMethod {
            access_flags: 0,
            name: b"method",
            descriptor: b"(JD)V",
            owner: b"Test",
            pool: &synthetic.pool,
            loader: &loader,
        };
        let table = frames_or_panic(
            frames(
                &synthetic.facts,
                &synthetic.canonical,
                &instance,
                &mut budget(),
            )
            .expect("a legal run is answered"),
        );
        let entry = entry_of(&table, 0);
        assert_eq!(
            entry.locals[0],
            Value::Ref(RefType::Named {
                name: b"Test".to_vec(),
                loader: Box::new(loader.clone()),
            }),
            "an instance method's local 0 is the receiver, named by the class file"
        );
        assert_eq!(entry.locals[1], Value::Long);
        assert_eq!(entry.locals[2], Value::Second);
        assert_eq!(entry.locals[3], Value::Double);
        assert_eq!(entry.locals[4], Value::Second);
        assert_eq!(entry.locals[5], Value::Top, "a slot no path wrote");

        let constructor = FrameMethod {
            access_flags: 0,
            name: b"<init>",
            descriptor: b"()V",
            owner: b"Test",
            pool: &synthetic.pool,
            loader: &loader,
        };
        let table = frames_or_panic(
            frames(
                &synthetic.facts,
                &synthetic.canonical,
                &constructor,
                &mut budget(),
            )
            .expect("a legal run is answered"),
        );
        assert_eq!(
            entry_of(&table, 0).locals[0],
            Value::UninitializedThis,
            "a constructor is entered with the uninitialized `this` 4.2 will flip"
        );
    }

    // -- D41, debug tables and the 4.2 boundary -------------------------------------------

    /// `multianewarray` pops one length per dimension: the count comes from the reader's operand
    /// fact, and the type it pushes is the class file's own.
    #[test]
    fn multianewarray_pops_one_length_per_dimension() {
        let expected = Value::Ref(RefType::Named {
            name: b"[[I".to_vec(),
            loader: Box::new(LoaderId("app".to_string())),
        });
        // Two dimensions: the arm pops two lengths and pushes the array, and the other arm
        // reaches the join with `null`, so the join's state says what the arm left.
        let (mut code, join) = diamond(
            &[0x03, 0x03],
            &[0xc5, 0x00, 0x09, 0x02],
            &[0x57, 0x57, 0x01],
        );
        code.push(0xb1); // return
        let fixture = fixture_body(&code, 4);
        let table = frames_or_panic(frames_of(&fixture));
        assert_eq!(
            entry_of(&table, join).stack,
            vec![expected.clone()],
            "two dimensions leave exactly the array on the stack"
        );

        // One dimension: exactly one length, and the same array type comes out.
        let (mut code, join) = diamond(&[0x03], &[0xc5, 0x00, 0x09, 0x01], &[0x57, 0x01]);
        code.push(0xb1); // return
        let fixture = fixture_body(&code, 4);
        let table = frames_or_panic(frames_of(&fixture));
        assert_eq!(entry_of(&table, join).stack, vec![expected]);
    }

    /// The fixtures of these tests carry no debug tables at all, which is what makes the
    /// derivation a derivation: nothing in the bytes says where a value is alive.
    #[test]
    fn a_body_without_any_debug_attribute_still_gets_frames() {
        let (mut code, join) = diamond(&[], &[0x03, 0x3c], &[0x01, 0x4c]);
        code.push(0xb1); // return
        let bytes = jarde_reader::classfile::test_class::single_method(52, 64, 4, &code);
        for name in [
            &b"StackMapTable"[..],
            b"LineNumberTable",
            b"LocalVariableTable",
            b"LocalVariableTypeTable",
        ] {
            assert!(
                !bytes.windows(name.len()).any(|window| window == name),
                "the fixture carries no {name:?}"
            );
        }
        let table = frames_or_panic(frames_of(&fixture_of(&bytes)));
        assert_eq!(
            entry_of(&table, join).locals[1],
            Value::Top,
            "the join is entered without any table saying what lives where"
        );
    }

    /// An uninitialized value may only be moved: consuming it as something else stops the body
    /// under the 4.1 boundary code, and never as a contradiction of its bytes.
    #[test]
    fn consuming_an_uninitialized_value_stops_the_body() {
        // 0  new Test (constant pool 2)   stack <- uninitialized(0)
        // 3  ifnull 6                     consumes it as a reference: 4.2's analysis, not this one
        // 6  return
        let fixture = fixture_body(
            &[
                0xbb, 0x00, 0x02, // new Test
                0xc6, 0x00, 0x03, // ifnull +3 -> 6
                0xb1, // return
            ],
            4,
        );
        let message = unproven(frames_of(&fixture));
        assert!(
            message.contains("uninitialized") && message.contains("4.2"),
            "the stop names the state and the slice that owns it: {message}"
        );

        // The same value moved by a pure stack operation is legal: `dup` then `pop` carries it.
        let fixture = fixture_body(
            &[
                0xbb, 0x00, 0x02, // new Test
                0x59, // dup
                0x57, // pop
                0xb1, // return
            ],
            4,
        );
        assert!(matches!(frames_of(&fixture), FrameOutcome::Frames(_)));
    }

    /// A block entered through an exception edge gets no invented state: the canonical edge
    /// aggregates a block's throw sites, and the per-throw-site input a handler needs is 4.2's.
    #[test]
    fn an_exception_edge_stops_the_body_at_the_4_2_boundary() {
        let synthetic = synthetic_body(
            vec![
                plain(0, 0x03), // iconst_0
                plain(1, 0x03), // iconst_0
                plain(2, 0x6c), // idiv (may raise)
                plain(3, 0xac), // ireturn
                plain(4, 0x57), // pop    (the handler entry)
                plain(5, 0xb1), // return
            ],
            vec![],
            4,
            6,
        );
        let mut facts = synthetic.facts.clone();
        facts.exception_handlers = vec![ExceptionHandlerFact {
            ordinal: 0,
            start_bci: 0,
            end_bci: 3,
            handler_bci: 4,
            catch_type_index: None,
        }];
        facts.exception_handler_count = 1;
        let canonical = canonical_of(&facts, 52);
        let synthetic = Synthetic {
            facts,
            pool: synthetic.pool,
            canonical,
        };
        let message = unproven(synthetic_frames(&synthetic, b"()V"));
        assert!(
            message.contains("exception edge") && message.contains("4.2"),
            "the stop names the edge and the slice that owns it: {message}"
        );
    }

    /// A body whose decode stopped early has the frames of its **reliable prefix**: the pass does
    /// not refuse a truncated graph, and it is the graph that says where the prefix stops — which
    /// is what makes the phase `Partial` in the report instead of `Completed`.
    #[test]
    fn a_truncated_body_gets_the_frames_of_its_prefix() {
        let facts = MethodCodeFacts::from_parts(
            64,
            4,
            ByteSpan::new(CODE_OFFSET, 5),
            vec![plain(0, 0x03), local(1, 0x3c, 1), plain(2, 0xb1)],
            Vec::new(),
            0,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::CodeBytes,
                },
                usage: UsageSnapshot::default(),
            },
            Some(BytecodeStop::Instructions {
                bci: 3,
                class_offset: CODE_OFFSET + 3,
                code: "classfile_bytecode_budget_exceeded".to_string(),
            }),
        );
        let canonical = canonical_of(&facts, 52);
        assert!(
            !canonical.completeness.is_complete(),
            "the fixture is a decoded prefix, not a whole body"
        );
        let synthetic = Synthetic {
            facts,
            pool: Vec::new(),
            canonical,
        };
        let table = frames_or_panic(synthetic_frames(&synthetic, b"()V"));
        assert_eq!(
            table.blocks().len(),
            1,
            "the frames cover the reliable prefix of the body"
        );
    }

    // -- The JVM's own bounds and the budget ----------------------------------------------

    /// The operand stack is bounded by the JVM's own slot ceiling, and a body that would grow past
    /// it contradicts itself instead of being analyzed at any size.
    #[test]
    fn the_stack_ceiling_is_enforced() {
        // `iconst_0` twice, then `dup2` repeated: every `dup2` adds two slots, so 32767 of them
        // ask for 65536 slots — one above the JVM's ceiling of 65535.
        let mut code = vec![0x03, 0x03];
        code.extend(std::iter::repeat_n(0x5c, 32_767));
        code.push(0xb1);
        let bytes = jarde_reader::classfile::test_class::single_method(52, 65_535, 0, &code);
        let fixture = fixture_of(&bytes);
        let message = inconsistent(frames_of(&fixture));
        assert!(
            message.contains("65535"),
            "the refusal names the ceiling it would pass: {message}"
        );
    }

    /// A cancelled request stops at a phase boundary and publishes nothing.
    #[test]
    fn a_cancelled_request_publishes_no_frames() {
        let (mut code, _) = diamond(&[], &[0x03, 0x3c], &[0x01, 0x4b]);
        code.push(0xb1); // return
        for phase in [Phase::Entry, Phase::Transfer, Phase::Publish] {
            let fixture = fixture_body(&code, 4);
            let loader = LoaderId("app".to_string());
            let method = method_of(&fixture, &loader, b"()V");
            let mut budget = budget();
            let _seam = checkpoint_seam(phase);
            let error = frames(&fixture.facts, &fixture.canonical, &method, &mut budget)
                .expect_err("a cancelled run has no outcome");
            assert!(
                matches!(error, Error::Cancelled { .. }),
                "cancelled at {phase:?}: {error:?}"
            );
        }
    }

    /// An exhausted `IrItems` dimension stops the run before the state it would have stored, and
    /// the dimension it names is the one this pass declares.
    #[test]
    fn an_exhausted_item_budget_stops_before_a_state_is_stored() {
        let (mut code, _) = diamond(&[], &[0x03, 0x3c], &[0x01, 0x4b]);
        code.push(0xb1); // return
        let fixture = fixture_body(&code, 4);
        let loader = LoaderId("app".to_string());
        let method = method_of(&fixture, &loader, b"()V");
        let mut budget = Budget::new(limits_with(|limits| limits.ir_items = 0));
        let error = frames(&fixture.facts, &fixture.canonical, &method, &mut budget)
            .expect_err("the entry state cannot be stored");
        let Error::BudgetExceeded {
            dimension, limit, ..
        } = error
        else {
            panic!("an exhausted dimension is a budget stop");
        };
        assert_eq!(dimension, BudgetDimension::IrItems);
        assert_eq!(limit, 0);
    }

    /// The pass bills the two dimensions its row declares and no other, and the states it stores
    /// are what the item charge counts: one frame slot per entered block, one per merge that
    /// changed a state, and one for the published table itself.
    #[test]
    fn the_pass_bills_exactly_its_declared_dimensions() {
        let (mut code, _) = diamond(&[], &[0x03, 0x3c], &[0x01, 0x4b]);
        code.push(0xb1); // return
        let fixture = fixture_body(&code, 4);
        let loader = LoaderId("app".to_string());
        let method = method_of(&fixture, &loader, b"()V");
        let mut budget = budget();
        let outcome = frames(&fixture.facts, &fixture.canonical, &method, &mut budget)
            .expect("a legal run is answered");
        let table = frames_or_panic(outcome);
        let usage = budget.usage();
        let entered = u64::try_from(table.blocks().len()).expect("a small fixture");
        let locals = u64::try_from(table.locals_slots()).expect("a small fixture");
        // Four entered blocks of four local slots, the one merge that changed the join's state,
        // and the published table.
        assert_eq!(entered, 4);
        assert_eq!(locals, 4);
        assert_eq!(usage.ir_items, entered * locals + locals + 1);
        assert_eq!(usage.ir_edges, 0, "this pass builds no edge of its own");
        assert!(usage.analysis_steps > 0, "the worklist ran");
        assert_eq!(
            table.deepest_stack(),
            0,
            "every block is entered empty here"
        );
    }
}
