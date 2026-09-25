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
//! * [`Value::Uninitialized`] carries its [`NewSite`] — the **canonical** block the `new` runs in
//!   plus its own BCI — so two allocations are different values and never one, including two
//!   clones of one shared subroutine whose `new`s map back to the same original BCI;
//!   [`Value::UninitializedThis`] is one token by definition and needs no site;
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
//! # Handler entries
//!
//! A block entered through an exception edge is entered with a state this pass **derives**, not
//! one it invents from the aggregated edge. The canonical edge of one exception-table record
//! stands for every throw site of its source block that the record covers, so the state is taken
//! per throw site: the block's transfer publishes the locals **each** of its throwing
//! instructions is entered with — the state at the instruction, before it takes effect — and the
//! edge contributes one input per such site, with a stack holding the single exception reference
//! (the record's catch type, or a conservative unknown reference for a catch-all). The handler's
//! entry state is the merge of those inputs with every other incoming one, and the block keeps an
//! enumerable [`LogicalInput`] per input so a later value-flow consumer counts logical
//! predecessors instead of raw edges.
//!
//! # Initialization
//!
//! A `new` pushes an uninitialized value whose token is the [`NewSite`] of the instruction — the
//! canonical node it runs in plus its own BCI — and a constructor's local 0 starts as
//! [`Value::UninitializedThis`]. Both are **tokens**, not types: a token is initialized by exactly
//! one instruction, the `invokespecial <init>` that constructs the object, and that call converts
//! **every alias** of the token it is given — in the locals and on the operand stack alike — into
//! an initialized reference. Nothing else converts a token, so two `new` sites stay two tokens
//! however their values are copied around, and a token only one path initialized is not the other
//! path's value.
//!
//! Whether a constructor call is **applicable** to the token it took is decided from the class
//! file's own names, never from "an `<init>` was called": `UninitializedThis` is converted by an
//! `<init>` of the class that declares this constructor or of its `super_class` — the two calls
//! JVMS 4.9.2 lets an instance initialization method make on its own `this` — and a `new`-site
//! token by an `<init>` of the class its own `new` allocates. Any other `<init>` call stops the
//! body: no conversion is defined for it, and whether such a body is legal is the verifier's
//! question rather than this pass's.
//!
//! Before that call, a token may be **moved** by a pure stack operation (`astore`/`aload`, the
//! `dup*` family, `pop*`, `swap`), and `UninitializedThis` may be stored through — as the target
//! of a `putfield` whose `Fieldref` **names the class being constructed**, which is the one
//! pre-initialization access JVMS 4.10.1.9 adds to the moves. That name is all this layer reads of
//! the rule: it holds no field table, so whether the class it names declares the field is a
//! question it cannot answer and does not ask. Every other consumer of a token stops the body. The
//! exception successor of a constructor call needs no rule of its own: its
//! input is the state **at** the call, taken before the call takes effect, so the conversion of
//! the normal successor cannot reach the handler.
//!
//! # The boundary that is left
//!
//! A token consumed where only an initialized reference is meaningful — `ifnull`, `areturn`,
//! `athrow`, `getfield`, an invocation that is not the applicable `<init>` — still stops the body:
//! this build neither decides whether such a body is legal (that is verification, and the report
//! states `verification` as `NotPerformed`) nor has a state to continue with, so it neither
//! guesses nor calls the bytes contradictory. That stop is [`FrameOutcome::Unsupported`], which
//! the driver reports under `ir_frame_deferred` — a different fact from the
//! [`FrameOutcome::Inconsistent`] a contradiction of the bytes produces. Silently skipping it, or
//! reporting it as a contradiction, is what the split exists to prevent.

use std::collections::BTreeMap;
use std::collections::VecDeque;

use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{
    Base, BaseType, CpEntryFacts, CpEntryKind, DescriptorComponent, DescriptorCursor,
    DescriptorKind, InstructionOperands, MethodCodeFacts, descriptor_facts,
};
use jarde_reader::error::{Error, Result};
use jarde_reader::view::LoaderId;

use crate::canonical::{
    CanonicalBlock, CanonicalBlockId, CanonicalCfg, CanonicalEdgeKind, CanonicalHandlerRow,
    CanonicalThrowSite,
};
use crate::method_ir::parameter_positions;

/// Stop code of a body this build does not state the frames of.
///
/// It means "an uninitialized value stands where only an initialized reference is meaningful, and
/// this pass neither converts it nor decides whether the bytes are legal", never "the bytecode is
/// wrong": the driver publishes the phases before this one and no `Frames` fact.
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
    /// Internal name of the declaring class's superclass — the class file's own `super_class` —
    /// or `None` for `java/lang/Object`, which declares none.
    ///
    /// A constructor's `UninitializedThis` is converted by an `<init>` of the class itself or of
    /// this class, the two calls JVMS 4.9.2 permits it, and that decision is the only reader.
    /// The class file names exactly these two: the pass reads no other header and holds no
    /// superclass chain.
    pub(crate) super_class: Option<&'a [u8]>,
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
pub enum RefType {
    /// The class file spells the type at the instruction or in a descriptor — an array creation,
    /// `checkcast`, `this`, a reference parameter — together with the loader anchor of the
    /// request, which the pass reads from its own input view and never from a resolution.
    Named {
        name: Vec<u8>,
        loader: Box<LoaderId>,
    },
    /// The slot holds a reference whose type these facts do not establish: a `ldc` constant that
    /// names no class at the instruction, or the merge of two differently named references.
    /// Conservative on purpose, and never confused with an unreadable slot or a basic type.
    Unknown,
}

/// Identity of one `new`: the **canonical** block the instruction runs in, plus its own BCI.
///
/// The raw BCI alone is not an identity in a normalized body. A shared subroutine is cloned once
/// per call site, and every clone maps back to the same original instructions — so two `new`s in
/// two clones carry one BCI between them, and an alias keyed by it would read the two allocations
/// as one token. The canonical block carries the call path that makes the clone, which is what
/// keeps the two apart. `UninitializedThis` needs no site: a constructor's own `this` is one
/// token by definition rather than one per allocation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NewSite {
    /// The canonical node the `new` runs in.
    pub(crate) block: CanonicalBlockId,
    /// BCI of the `new` instruction, inside the original code that node stands for.
    pub(crate) bci: u32,
}

impl NewSite {
    /// The canonical node the `new` runs in.
    pub fn block(&self) -> &CanonicalBlockId {
        &self.block
    }

    /// BCI of the `new` instruction.
    pub fn bci(&self) -> u32 {
        self.bci
    }
}

/// One slot's state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
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
    /// An uninitialized value whose `new` runs at this site: the identity every alias of the
    /// value — and only that value — shares.
    Uninitialized {
        new_site: NewSite,
    },
    /// The return address a `jsr` pushes, which `astore`/`aload` may carry like a reference.
    ReturnAddress,
}

impl Value {
    /// Slots this value occupies where it is the first slot of the value.
    pub(crate) fn slots(&self) -> usize {
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
    /// produces nothing: what it does produce is the initialization of the token it was given,
    /// which [`constructor_call`] converts when this call is the applicable one for that token.
    Invoke { receiver: bool },
    /// `new`: push an uninitialized value whose site is this instruction, in this block.
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
///
/// The type is crate-visible because a later pass of the same pipeline consumes a helper of this
/// module that can answer all three ([`caught_reference`]), and the split is what that caller has
/// to keep apart: collapsing it on the way out would report a boundary of this build as a
/// contradiction of the body, or the other way round.
pub(crate) enum Problem {
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
    /// Trace sink of the replay [`block_touches`] performs, `None` in every state this pass
    /// computes, merges, compares or publishes: the trace is not part of a state's meaning and
    /// no comparison of two states ever sees a sink. It is the one way 4.3 gets the slot
    /// accesses of an instruction out of the dense table below instead of classifying opcodes a
    /// second time, and it exists only while that replay runs.
    touches: Option<Vec<SlotTouch>>,
}

/// The region of a frame one slot access names.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum SlotRegion {
    /// A local slot, by `max_locals` index.
    Local,
    /// The operand stack, by slot depth from the bottom.
    Stack,
}

/// One slot access of one instruction, recorded where this pass performs it.
///
/// A read carries the width of the value it took, a write the class of the value it stored, and
/// both name the **first** slot of the value only: the upper half of a category-2 value is never
/// an access of its own, which is what lets 4.3 hold one SSA value for the two slots. Recording
/// at the frame's own mutation points is deliberate — the shapes of `pop2`, `dup2_x2` and their
/// family follow the values on the stack, so a second, value-independent classification of the
/// opcodes would be wrong for exactly those forms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SlotTouch {
    pub(crate) region: SlotRegion,
    /// Local index, or the stack depth of the value's first slot.
    pub(crate) slot: u32,
    /// Slots the value occupies: 1, or 2 for a category-2 value.
    pub(crate) width: u32,
    /// `true` for a write, `false` for a read.
    pub(crate) write: bool,
    /// Class of the value a write stores; `None` for a read, whose class the reader of the SSA
    /// takes from the definition it resolved.
    pub(crate) value: Option<Value>,
}

/// What one instruction of a block touches, in execution order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InstructionTouches {
    pub(crate) bci: u32,
    /// The effective opcode, as 1.2 records it for a `wide` form.
    pub(crate) opcode: u8,
    /// The operand-stack depth, in slots, the block's state has **after** this instruction. It is
    /// the frame's own length, so the discards an instruction performs without reading anything —
    /// `athrow` clears the stack, a `return` ends the block with it — are stated exactly and not
    /// inferred from the accesses.
    pub(crate) stack_after: u32,
    pub(crate) accesses: Vec<SlotTouch>,
}

/// What one replay of a block produced.
#[derive(Debug)]
pub(crate) enum TouchesOutcome {
    Touches(Vec<InstructionTouches>),
    /// This build does not state the accesses of this body: the same boundary [`frames`] reports,
    /// reached again by the replay.
    Unsupported {
        message: String,
    },
    /// The body contradicts itself, or the replay of a **published** entry state disagrees with
    /// the frames this run published for it. The second is a defect of this build's own artifact
    /// and not a claim about the bytes, and it is reported as a contradiction rather than
    /// silently absorbed.
    Inconsistent {
        message: String,
    },
}

/// One slot of the state a method's own entry block is entered with.
///
/// The caller's own contribution is a participant of that block's entry phis like any incoming
/// edge: a body that branches back to BCI 0 merges the caller's state with the incoming one, and
/// the edge alone does not state the caller's half. This is that half, as plain data, so the SSA
/// does not have to name a frame state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EntrySlot {
    pub(crate) region: SlotRegion,
    /// Local index, or 0 for the one value of the entry state's operand stack.
    pub(crate) slot: u32,
    pub(crate) value: Value,
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
        if self.touches.is_some() {
            let slot = depth - value.slots();
            self.record(
                SlotRegion::Stack,
                slot,
                value.slots(),
                true,
                Some(value.clone()),
            );
        }
        if value.slots() == 2 {
            self.stack.push(value);
            self.stack.push(Value::Second);
        } else {
            self.stack.push(value);
        }
        Ok(())
    }

    /// Records one access when a replay is tracing this frame; a no-op in every other run.
    fn record(
        &mut self,
        region: SlotRegion,
        slot: usize,
        width: usize,
        write: bool,
        value: Option<Value>,
    ) {
        if let Some(touches) = self.touches.as_mut() {
            touches.push(SlotTouch {
                region,
                slot: u32::try_from(slot).unwrap_or(u32::MAX),
                width: u32::try_from(width).unwrap_or(u32::MAX),
                write,
                value,
            });
        }
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
        let value = match top {
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
        }?;
        if self.touches.is_some() {
            let slot = self.stack.len();
            self.record(SlotRegion::Stack, slot, value.slots(), false, None);
        }
        Ok(value)
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
    /// interpreting it. A token of a `new` or of an uninitialized `this` reaches its initialized
    /// state through the constructor call that constructs it and through nothing else, so every
    /// other consumer of one stops the body — the run neither guesses a type for it nor calls the
    /// body inconsistent.
    fn pop_ty(&mut self, ty: Ty, bci: u32, opcode: u8, moves: bool) -> Norm<Value> {
        let value = self.take(bci, opcode)?;
        if value.is_uninitialized() {
            if !moves || ty != Ty::Ref {
                return Err(Problem::Unproven(format!(
                    "`{opcode:#04x}` at BCI {bci} consumes the uninitialized value {value:?} as \
                     something other than a moved reference; a token is moved, or converted by the \
                     `<init>` call that constructs it, and this instruction does neither"
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

    /// Removes the receiver of an `invoke*`.
    ///
    /// This is the one operand a token may be: the applicable `invokespecial <init>` takes the
    /// uninitialized value it converts, and the caller decides from the pool entry's own name
    /// whether the call it took it for is that call. Every other consumer of a token is refused
    /// here exactly as [`Frame::pop_ty`] refuses it, and an initialized reference is the ordinary
    /// answer.
    fn take_receiver(&mut self, bci: u32, opcode: u8) -> Norm<Value> {
        let value = self.take(bci, opcode)?;
        if value.is_uninitialized() || Ty::Ref.accepts(&value) {
            return Ok(value);
        }
        inconsistent(format!(
            "`{opcode:#04x}` at BCI {bci} needs a Ref where the stack holds {value:?}"
        ))
    }

    /// Removes the target reference of a field access, under the one rule that lets an
    /// uninitialized `this` be given to a `putfield` (JVMS 4.10.1.9).
    ///
    /// `own_field` is the caller's decision that the restricted form holds: the `Fieldref` **names
    /// the class being constructed** — the class whose instance initialization method this body
    /// is, since nothing but that entry makes an uninitialized `this`. That name is the whole of
    /// what this layer reads: it holds no field table, so a class file that declares no field at
    /// all passes here whenever its `Fieldref` names the class, and whether the field is really
    /// declared is left to the verifier. Only then may `UninitializedThis` stand here, and it is
    /// the only token that may: a value a `new` produced is constructed by its own `<init>` call,
    /// and JVMS 4.10.1.9 gives it no field assignment. Everything else stops the body under the
    /// boundary code instead of being reported as a contradiction of bytes this pass cannot call
    /// illegal — being certain of *that* is the verifier's work.
    fn pop_field_target(&mut self, bci: u32, opcode: u8, own_field: bool) -> Norm<()> {
        let value = self.take(bci, opcode)?;
        match value {
            Value::UninitializedThis if own_field => Ok(()),
            Value::UninitializedThis => Err(Problem::Unproven(format!(
                "`{opcode:#04x}` at BCI {bci} stores the uninitialized `this` through a `Fieldref` \
                 that does not name the class being constructed; the pre-initialization `putfield` \
                 of JVMS 4.10.1.9 reaches only a `Fieldref` that names it"
            ))),
            value if value.is_uninitialized() => Err(Problem::Unproven(format!(
                "`{opcode:#04x}` at BCI {bci} consumes the uninitialized value {value:?} as the \
                 target of a field access; a value a `new` produced is initialized by its own \
                 `<init>` call, and JVMS 4.10.1.9 gives it no field assignment"
            ))),
            value if Ty::Ref.accepts(&value) => Ok(()),
            value => inconsistent(format!(
                "`{opcode:#04x}` at BCI {bci} needs a Ref where the stack holds {value:?}"
            )),
        }
    }

    /// Converts every alias of one token into the initialized reference a constructor call
    /// produced — in the locals and on the operand stack alike.
    ///
    /// Equality **is** the token's identity ([`Value::Uninitialized`] carries its [`NewSite`]),
    /// which is what keeps two allocations apart: only the aliases of the token this call was
    /// given are converted, while a value another `new` produced, or the `UninitializedThis` of
    /// a constructor, is not this token and is left as it stands.
    ///
    /// How many slots the conversion reached is not answered, because no rule of this pass reads
    /// that number: the receiver the call was given is already out of the frame, so a token whose
    /// only alias it was converts none at all, and what the call leaves behind is the frame's own
    /// new state.
    ///
    /// The conversion is a definition of every slot it changes: a value-flow consumer sees the
    /// alias holding a new value afterwards rather than a slot whose class changed under it, so
    /// a traced replay records one write per converted slot.
    fn convert_token(&mut self, token: &Value, initialized: &Value) {
        if self.touches.is_some() {
            let mut converted: Vec<(SlotRegion, usize)> = Vec::new();
            for (index, slot) in self.locals.iter_mut().enumerate() {
                if slot == token {
                    *slot = initialized.clone();
                    converted.push((SlotRegion::Local, index));
                }
            }
            for (depth, slot) in self.stack.iter_mut().enumerate() {
                if slot == token {
                    *slot = initialized.clone();
                    converted.push((SlotRegion::Stack, depth));
                }
            }
            for (region, slot) in converted {
                self.record(
                    region,
                    slot,
                    initialized.slots(),
                    true,
                    Some(initialized.clone()),
                );
            }
            return;
        }
        for slot in self.locals.iter_mut().chain(self.stack.iter_mut()) {
            if slot == token {
                *slot = initialized.clone();
            }
        }
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
            Value::UninitializedThis | Value::Uninitialized { .. } if ty == Ty::Ref => {
                self.record(
                    SlotRegion::Local,
                    usize::from(index),
                    value.slots(),
                    false,
                    None,
                );
                Ok(value)
            }
            Value::UninitializedThis | Value::Uninitialized { .. } => {
                Err(Problem::Unproven(format!(
                    "`{opcode:#04x}` at BCI {bci} reads the uninitialized value {value:?} from local \
                 {index} as a {ty:?}; a token is read as a reference, moved, or converted by the \
                 `<init>` call that constructs it"
                )))
            }
            value if ty.accepts(&value) => {
                self.record(
                    SlotRegion::Local,
                    usize::from(index),
                    value.slots(),
                    false,
                    None,
                );
                Ok(value)
            }
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
        if self.touches.is_some() {
            self.record(
                SlotRegion::Local,
                start,
                value.slots(),
                true,
                Some(value.clone()),
            );
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
/// without a class hierarchy: two names that are not one class merge to a conservative unknown
/// reference ([`merged_reference`] is where the two spellings one class reaches this table in are
/// told apart from two classes), `null` merges with a named reference into that reference, and the
/// null type with itself stays the null type.
/// The merge of two locals arrays entering one block: slot by slot, under the rule that a local
/// this pass cannot decide becomes [`Value::Top`].
///
/// Two uninitialized values merge to `Top` here like any other disagreement, and that stays
/// deliberate now that the exception inputs are computed: the token of a slot is either the same
/// on both paths — and then the equality above already returned it — or the paths hold different
/// states. After such a merge point, reading that slot is illegal on the path whose token is
/// missing or uninitialized, so failing *at the read* is the precise place to fail, and a body
/// that differs only in a local it never reads stays analyzable. The operand stack answers the
/// same question differently on purpose ([`merge_stack`]): a stack disagreement is a shape the
/// next instruction would misinterpret, while a dead local is not.
fn merge_local(left: &Value, right: &Value) -> Value {
    if left == right {
        return left.clone();
    }
    match (left, right) {
        (Value::Top, _) | (_, Value::Top) | (Value::Second, _) | (_, Value::Second) => Value::Top,
        (Value::Null, Value::Ref(other)) | (Value::Ref(other), Value::Null) => {
            Value::Ref(other.clone())
        }
        (Value::Ref(left), Value::Ref(right)) => Value::Ref(merged_reference(left, right)),
        _ => Value::Top,
    }
}

/// The type two references merge into: the class both of them name, or the conservative unknown
/// reference when they do not name one.
///
/// Equality alone is not the question, because one class reaches this table in **two spellings**: a
/// class entry states its own internal name (`Test`, `Lazy$Holder`), while a field or method
/// descriptor states the type as the descriptor slice it is (`LTest;`, `LLazy$Holder;`). The two are
/// one type, and every reader of the published table treats them as one — the oracle projects both
/// into the internal name for exactly this reason ([`crate::frame_oracle::internal_name`]), and so
/// does the recovery layer's own comparison of a receiver with the class file's owner. Comparing the
/// raw bytes here would answer `Unknown` for a merge of the two spellings and drop a class the class
/// file's own `StackMapTable` states for the slot, which is a fact this table carries rather than
/// loses: the merge of the two writes of `local0 = Lazy.h; … local0 = new Holder();` is a
/// `Lazy$Holder`, not an unknown reference.
fn merged_reference(left: &RefType, right: &RefType) -> RefType {
    match (left, right) {
        (
            RefType::Named { name, loader },
            RefType::Named {
                name: other,
                loader: other_loader,
            },
        ) if loader == other_loader && class_of(name) == class_of(other) => left.clone(),
        _ => RefType::Unknown,
    }
}

/// One named reference's class, with the descriptor wrapping off.
///
/// `LLazy$Holder;` and `Lazy$Holder` are one class. An array descriptor (`[LLazy$Holder;`) is a type
/// of its own and keeps its spelling, as does every name the class file did not wrap in `L…;`.
fn class_of(name: &[u8]) -> &[u8] {
    match name {
        [b'L', rest @ .., b';'] => rest,
        other => other,
    }
}

/// The merge of two operand stacks entering one block.
///
/// The stack is the half that must agree: depth and slot classes decide whether the body can hold
/// a value at all, so a disagreement here is `ir_frame_inconsistent` and not a `Top`.
/// Uninitialized values are the one case this build does not decide: two tokens are two values,
/// and a token reaches its initialized state only through the constructor call that was given
/// *that* token, so a merge of them has no state that keeps either path's meaning. They stop the
/// body under their own code instead of being reported as a contradiction, or being merged into a
/// token no path holds.
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
                merged.push(Value::Ref(merged_reference(left, right)));
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
/// the stack rule, charged before the merged state is allocated.
///
/// The merged state has exactly the shape of `current`: one slot per local, and a stack of the
/// same depth, because a disagreement in either is a contradiction of the body and not a merge.
/// So its slots are charged before the vectors that hold them are built — including when the merge
/// turns out to be `current` itself, because that attempt allocated a state too.
fn merge_frame(
    budget: &mut Budget,
    current: &Frame,
    incoming: &Frame,
    block: &CanonicalBlockId,
) -> Norm<Frame> {
    charge_frame(budget, current)?;
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
    Ok(Frame {
        locals,
        stack,
        touches: None,
    })
}

/// One parameter of a method descriptor: its slot class, and the type the descriptor spells for a
/// reference.
struct Param {
    /// The slot class the parameter occupies.
    ty: Ty,
    /// The internal name the descriptor spells, for a reference parameter.
    name: Option<Vec<u8>>,
}

/// One descriptor component as the frame states it: the slot class of its value, and — for a
/// reference — the name [`RefType::Named`] carries.
///
/// The component is the reader's own reading of the production ([`DescriptorComponent`]); what this
/// layer adds is the **slot class**, which is the frame's vocabulary and not the descriptor's: the
/// four int-shaped primitives are one class here ([`Ty::Int`]), and **an array is a reference
/// whatever its element type** — `[J` is not a `long`, and `[[D` is not a `double`. The name is the
/// bytes the component spans, exactly as the class file spells the type (`Ljava/lang/String;`,
/// `[I`, `[[Ljava/lang/String;`), which is the form the frames keep a named reference in.
fn component_frame_type(
    descriptor: &[u8],
    component: &DescriptorComponent,
) -> Norm<(Ty, Option<Vec<u8>>)> {
    let base = match component.base() {
        Base::Primitive(BaseType::Float) => Ty::Float,
        Base::Primitive(BaseType::Long) => Ty::Long,
        Base::Primitive(BaseType::Double) => Ty::Double,
        // `boolean`, `byte`, `char`, `short` and `int` are one slot class: the descriptor states
        // which of them the *parameter* is, and a frame cannot (the note on [`Ty`]).
        Base::Primitive(_) => Ty::Int,
        Base::Object(_) => Ty::Ref,
    };
    let ty = if component.is_array() { Ty::Ref } else { base };
    if ty != Ty::Ref {
        return Ok((ty, None));
    }
    let bytes = component.bytes(descriptor).ok_or_else(|| {
        Problem::Inconsistent(format!(
            "the descriptor component at [{}..+{}] is outside the descriptor it was read from",
            component.span().start,
            component.span().length
        ))
    })?;
    Ok((ty, Some(bytes.to_vec())))
}

/// Parses one field descriptor at the start of `bytes`.
///
/// Returns the slot class, the number of bytes the descriptor used, and — for a reference — the
/// internal name the descriptor spells, which is what [`RefType::Named`] carries. A descriptor the
/// class-file format does not allow is a contradiction of the body: the reader keeps the bytes and
/// does not validate them, so this is where a malformed one is refused.
fn parse_field_type(bytes: &[u8]) -> Norm<(Ty, usize, Option<Vec<u8>>)> {
    let mut cursor = DescriptorCursor::new(bytes);
    let component = cursor
        .field_type()
        .map_err(|error| Problem::Inconsistent(error.to_string()))?;
    let (ty, name) = component_frame_type(bytes, &component)?;
    let length = usize::try_from(component.span().length).map_err(|_| {
        Problem::Inconsistent("a descriptor component is longer than usize".to_string())
    })?;
    Ok((ty, length, name))
}

/// Parses a method descriptor `(parameters)return`.
fn parse_method_descriptor(bytes: &[u8]) -> Norm<(Vec<Param>, Option<Param>)> {
    let facts = descriptor_facts(bytes, DescriptorKind::Method)
        .map_err(|error| Problem::Inconsistent(error.to_string()))?;
    let mut parameters = Vec::with_capacity(facts.parameters().len());
    for component in facts.parameters() {
        let (ty, name) = component_frame_type(bytes, component)?;
        parameters.push(Param { ty, name });
    }
    let returns = match facts.result() {
        Some(component) => {
            let (ty, name) = component_frame_type(bytes, component)?;
            Some(Param { ty, name })
        }
        None => None,
    };
    Ok((parameters, returns))
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

/// The class one `Fieldref` operand names as the field's declaring class.
///
/// The restricted `putfield` of JVMS 4.10.1.9 is decided on this name, and the kind is the one
/// [`pool_field_descriptor`] already required of a field access.
fn pool_field_owner<'a>(
    method: &'a FrameMethod<'_>,
    operands: &InstructionOperands,
    bci: u32,
    opcode: u8,
) -> Norm<&'a [u8]> {
    let entry = pool_entry(method.pool, operands, bci, opcode)?;
    match &entry.kind {
        CpEntryKind::FieldRef { owner, .. } => Ok(&owner.0),
        other => inconsistent(format!(
            "`{opcode:#04x}` at BCI {bci} names a {} entry where a field reference is required",
            cp_kind_name(other)
        )),
    }
}

/// The class one invocation's entry names as the owner of a method called `<init>`, whatever the
/// opcode is.
///
/// The name is the entry's own fact. Every invocation opcode that names a method reference may
/// name one called `<init>` — the bytes decide that, not the opcode — where `invokedynamic` names a
/// call site of its own kind and is no such entry. The kinds are the pairing
/// [`pool_method_descriptor`] already required, and a non-method entry answers `None` here so that
/// the descriptor check keeps naming what is wrong with it.
fn init_named_target<'a>(
    method: &'a FrameMethod<'_>,
    operands: &InstructionOperands,
    bci: u32,
    opcode: u8,
) -> Norm<Option<&'a [u8]>> {
    let entry = pool_entry(method.pool, operands, bci, opcode)?;
    match &entry.kind {
        CpEntryKind::MethodRef { owner, name, .. }
        | CpEntryKind::InterfaceMethodRef { owner, name, .. }
            if name.0.as_slice() == b"<init>" =>
        {
            Ok(Some(&owner.0))
        }
        _ => Ok(None),
    }
}

/// The class an `invokespecial <init>` constructs, when the entry it names is a constructor call
/// at all.
///
/// `None` says "no initialization conversion is defined here": the instruction is not an
/// `invokespecial`, or the name the constant pool gives the method is not `<init>` — the name is
/// the entry's own fact, and an `invokevirtual` of a method called `<init>` is not a constructor
/// call here, since the one transition this pass has for a call named `<init>` belongs to the
/// `invokespecial` that constructs a token. The kind is the pairing [`pool_method_descriptor`]
/// already required.
fn constructor_target<'a>(
    method: &'a FrameMethod<'_>,
    operands: &InstructionOperands,
    bci: u32,
    opcode: u8,
) -> Norm<Option<&'a [u8]>> {
    if opcode != 0xb7 {
        return Ok(None);
    }
    init_named_target(method, operands, bci, opcode)
}

/// The class one `new` site allocates, read from the instruction the site names.
///
/// The class is a fact of the **instruction**: two clones of one shared subroutine map back to
/// one `new`, allocate one class, and are told apart by the token's other half ([`NewSite::block`]).
/// Storing the name in the value would put a second copy of it in every alias — state the frame
/// carries, merges and would have to charge for — so it is read here, from the decoded `new` the
/// site points at, on the one instruction that needs it: the constructor call that converts the
/// token.
fn new_site_class(
    method: &FrameMethod<'_>,
    facts: &MethodCodeFacts,
    new_site: &NewSite,
    bci: u32,
) -> Norm<Vec<u8>> {
    let operands = facts.operands();
    let position = facts
        .instructions
        .partition_point(|instruction| instruction.bci < new_site.bci);
    let (Some(instruction), Some(operands)) =
        (facts.instructions.get(position), operands.get(position))
    else {
        return inconsistent(format!(
            "the uninitialized value a constructor call at BCI {bci} converts names the `new` at \
             BCI {}, which is not an instruction with operand facts of this body",
            new_site.bci
        ));
    };
    let effective = operands.effective_opcode;
    if instruction.bci != new_site.bci || effective != 0xbb {
        return inconsistent(format!(
            "the uninitialized value a constructor call at BCI {bci} converts names a `new` at \
             BCI {}, where the decoded body holds `{effective:#04x}`",
            new_site.bci
        ));
    }
    pool_class_name(method, operands, new_site.bci, effective)
}

/// The initialization conversion an `invokespecial <init>` performs on the token it was given.
///
/// The call is **applicable** to one of the two tokens only when the class file says so.
/// `UninitializedThis` is converted by an `<init>` of the class that declares this constructor or
/// of its `super_class` — the two calls JVMS 4.9.2 lets an instance initialization method make on
/// its own `this` — and a token from `new` by an `<init>` of the class that `new` allocated. Any
/// other constructor call, and any invocation that does not name an `<init>` at all, stops the
/// body: no conversion is defined for it, and whether such a body is legal is the verifier's
/// question, which this pass does not answer.
///
/// On an applicable call the token becomes an initialized reference to the class the conversion
/// is *of*: the class being constructed for `this` — its own class, whichever of the two allowed
/// calls did it — and the allocated class for a `new` site. Every alias of the token is
/// converted, in the locals and on the stack alike, and no other value is touched: the exception
/// successor of this call keeps the token, because its input was taken before the call.
fn constructor_call(
    method: &FrameMethod<'_>,
    facts: &MethodCodeFacts,
    operands: &InstructionOperands,
    bci: u32,
    opcode: u8,
    token: &Value,
    frame: &mut Frame,
) -> Norm<()> {
    let Some(class) = constructor_target(method, operands, bci, opcode)? else {
        return Err(Problem::Unproven(format!(
            "`{opcode:#04x}` at BCI {bci} takes the uninitialized value {token:?} as its receiver \
             and does not name an `<init>` of an `invokespecial`; only the constructor call that \
             constructs a value may take one"
        )));
    };
    let initialized = match token {
        Value::UninitializedThis => {
            if class != method.owner && Some(class) != method.super_class {
                return Err(Problem::Unproven(format!(
                    "`{opcode:#04x}` at BCI {bci} invokes the `<init>` of `{}` on the \
                     uninitialized `this` of `{}`; the calls JVMS 4.9.2 lets a constructor make \
                     on its own `this` are its own class's and its direct superclass's",
                    String::from_utf8_lossy(class),
                    String::from_utf8_lossy(method.owner)
                )));
            }
            named(method.owner.to_vec(), method)
        }
        Value::Uninitialized { new_site } => {
            let allocated = new_site_class(method, facts, new_site, bci)?;
            if allocated.as_slice() != class {
                return Err(Problem::Unproven(format!(
                    "`{opcode:#04x}` at BCI {bci} invokes the `<init>` of `{}` on the value the \
                     `new` at BCI {} allocated, which is a `{}`; only the allocated class's own \
                     `<init>` constructs it",
                    String::from_utf8_lossy(class),
                    new_site.bci,
                    String::from_utf8_lossy(&allocated)
                )));
            }
            named(allocated, method)
        }
        // The caller asks this about an uninitialized value alone.
        other => {
            return inconsistent(format!(
                "BCI {bci} asks for the initialization conversion of {other:?}, which is not an \
                 uninitialized value"
            ));
        }
    };
    frame.convert_token(token, &initialized);
    Ok(())
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
/// Applies one instruction of a block.
///
/// `block` is the canonical node the instruction runs in, which is the context a value's identity
/// needs: a `new` is told apart from the same original `new` in another clone of its subroutine by
/// the node, not by the BCI ([`NewSite`]). `facts` is the decoded body beside the operand facts,
/// read when a constructor call needs the class its receiver's `new` allocated — a fact of the
/// instruction, which the frame does not carry.
fn apply(
    method: &FrameMethod<'_>,
    facts: &MethodCodeFacts,
    entry: Entry,
    block: &CanonicalBlockId,
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
        Stack::Constant(effect) => {
            apply_constant(method, facts, effect, block, bci, operands, frame)
        }
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
    facts: &MethodCodeFacts,
    effect: PoolEffect,
    block: &CanonicalBlockId,
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
                    // JVMS 4.10.1.9's one addition to the moves: an instance initialization method
                    // may assign a field through the `this` it has not initialized yet. Holding an
                    // uninitialized `this` at all means this body *is* such a method — it is the
                    // entry state of an `<init>` and nothing else makes one — so the rule's
                    // condition reduces, at this layer, to the owner name the `putfield`'s
                    // `Fieldref` gives: the class file's own name is the class being constructed.
                    // Whether that class declares the field is not read here; this layer holds no
                    // field table.
                    let own_field =
                        pool_field_owner(method, operands, bci, opcode)? == method.owner;
                    frame.pop_field_target(bci, opcode, own_field)?;
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
            // A method called `<init>` is a constructor call whichever invocation opcode names it,
            // and this pass has exactly one transition for a constructor call: the initialization
            // conversion the applicable `invokespecial <init>` performs on the token it constructs.
            // Under any other opcode the call has no transition here, so the body stops at the
            // boundary rather than reading an ordinary invocation into the entry — `invokevirtual`
            // and `invokestatic` of a method named `<init>` get the same answer as an
            // `invokespecial <init>` that constructs nothing, which is the reading this layer
            // already gives every `<init>` it cannot convert.
            if opcode != 0xb7
                && let Some(class) = init_named_target(method, operands, bci, opcode)?
            {
                return Err(Problem::Unproven(format!(
                    "`{opcode:#04x}` at BCI {bci} invokes `{}`.<init>, an entry named `<init>` that \
                     is not an `invokespecial`; the only transition a constructor call has here is \
                     the initialization conversion of the token an `invokespecial <init>` \
                     constructs, and whether such a call is legal at all is the verifier's question",
                    String::from_utf8_lossy(class)
                )));
            }
            // The arguments sit above the receiver and are consumed in reverse order; the
            // descriptor is what decides how many of them there are and how wide.
            for param in params.iter().rev() {
                frame.pop_ty(param.ty, bci, opcode, false)?;
            }
            if receiver {
                // An `invokespecial <init>` is the one instruction that may take an uninitialized
                // receiver, and its normal completion is what converts every alias of the token it
                // was given. Whether this call is the applicable one, and what it converts the
                // token to, is decided from the class file's own names.
                let target = frame.take_receiver(bci, opcode)?;
                if target.is_uninitialized() {
                    constructor_call(method, facts, operands, bci, opcode, &target, frame)?;
                } else if constructor_target(method, operands, bci, opcode)?.is_some() {
                    // An `<init>` call on a receiver that is already initialized. JVMS 4.9.2 says
                    // an instance initialization method must never be invoked on an initialized
                    // instance, and this pass has exactly one transition for a constructor call —
                    // the conversion of a token — which does not apply here. Carrying on as if it
                    // were an ordinary `void` call would be this pass reading a meaning into bytes
                    // it cannot vouch for, so the body stops at the boundary instead.
                    return Err(Problem::Unproven(format!(
                        "`{opcode:#04x}` at BCI {bci} invokes an `<init>` on {target:?}, which is \
                         not an uninitialized value; the only transition a constructor call has \
                         here is the initialization of the value it constructs, and whether such a \
                         call is legal at all is the verifier's question"
                    )));
                }
            }
            match returns {
                Some(returns) => frame.push(value_of(returns.ty, returns.name, method), bci),
                None => Ok(()),
            }
        }
        PoolEffect::New => frame.push(
            Value::Uninitialized {
                new_site: NewSite {
                    block: block.clone(),
                    bci,
                },
            },
            bci,
        ),
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

/// One throwing instruction of a block, with the locals it is entered with.
///
/// The locals are the state **at** the instruction — before it takes effect — because that is the
/// state its own exception successor is entered with. The block's exit state is not that state:
/// instructions after this one may still have written the locals, and instructions before it may
/// not have written them yet.
#[derive(Clone, Debug)]
struct ThrowPoint {
    /// BCI of the instruction that may throw.
    bci: u32,
    /// Exception-table ordinals whose protected range covers that instruction, in declaration
    /// order, as the canonical throw site of that BCI states them.
    handlers: Vec<u32>,
    /// The locals the instruction sees, before it takes effect.
    locals: Vec<Value>,
}

/// What one run of a block leaves behind.
#[derive(Debug)]
struct Transfer {
    /// The exit state: the entry state with every instruction of the block applied in order.
    exit: Frame,
    /// One entry per throw site of the block, in the order the block runs them.
    throw_points: Vec<ThrowPoint>,
}

/// The exit state of one node and the state at each of its throw sites.
///
/// A throw site is what the canonical graph says may throw here — the same records the exception
/// edges are built from — so the transfer reads the site list instead of guessing "may throw" a
/// second time, and a site's own handlers decide which exception edge it feeds.
fn transfer_block(
    method: &FrameMethod<'_>,
    canonical: &CanonicalCfg,
    block: &CanonicalBlock,
    facts: &MethodCodeFacts,
    entry: Frame,
    budget: &mut Budget,
) -> Norm<Transfer> {
    let mut frame = entry;
    let operands = facts.operands();
    let sites: BTreeMap<u32, &CanonicalThrowSite> = canonical
        .throw_sites
        .iter()
        .filter(|site| site.block == block.id)
        .map(|site| (site.bci, site))
        .collect();
    let mut throw_points = Vec::with_capacity(sites.len());
    for index in instruction_indices(block, facts)? {
        let instruction = &facts.instructions[index];
        let operands = &operands[index];
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let row = TABLE[usize::from(operands.effective_opcode)];
        if let Some(site) = sites.get(&instruction.bci) {
            // A site no handler record covers has no reader at all: the exception inputs of this
            // block are taken per *covered* site, so a site whose own record list is empty feeds
            // no exception edge of any block. Its snapshot would be slot storage nobody reads, so
            // nothing is retained for it and nothing is charged — the product of sites and locals
            // this transfer holds is the product of the sites that really have a consumer.
            if !site.handlers.is_empty() {
                // Charged **before** the clone: one item per local slot the snapshot holds. A
                // block with many throw sites holds every one of these at once, so this is the
                // charge that makes the budget bound that product.
                charge_slots(budget, frame.locals.len())?;
                // The snapshot is taken *before* the instruction is applied: this is the state
                // the site's own exception successor is entered with, and the state the alias
                // conversion of a constructor call must not be able to reach.
                throw_points.push(ThrowPoint {
                    bci: instruction.bci,
                    handlers: site.handlers.clone(),
                    locals: frame.locals.clone(),
                });
            }
        }
        apply(
            method,
            facts,
            row,
            &block.id,
            instruction.bci,
            operands,
            &mut frame,
        )?;
    }
    Ok(Transfer {
        exit: frame,
        throw_points,
    })
}

/// One input an exception edge carries into its handler.
struct ExceptionInput {
    /// BCI of the throwing instruction the input is taken at, for an input a throw site hands
    /// over; `None` for the input of a record whose protected range covers no throwing
    /// instruction of its source block, which no site anchors.
    bci: Option<u32>,
    /// The handler's entry state along this input.
    frame: Frame,
}

/// The frames one exception edge carries into its handler: **one input per throw site** of the
/// source block that the record covers, or one input taken from the block's own exit when the
/// record covers no site at all.
///
/// The canonical edge aggregates a block's throw sites into a single edge — the raw graph keeps
/// one edge per `(block, ordinal)` pair — and this is where that aggregation is undone: each site
/// contributes the locals it is entered with plus a stack holding the single exception reference,
/// which is the shape a handler is entered with (JVMS 4.10.1.6). The reference is the record's
/// catch type when the class file names one, and a conservative unknown reference for a catch-all
/// record, whose type no fact of this request establishes.
///
/// A record whose range intersects the block but covers **no** throwing instruction of it — the
/// shape `javac --release 8` emits for a `try` whose body cannot raise — is still a row of the
/// table and still an edge of the graph, and a handler with no input at all is a handler no later
/// pass could present. Nothing can be raised at such a block, so no state is pinned by the bytes;
/// the input is the one state this block really derives, its own exit, with the same caught
/// reference on the stack. It names no throw site, and [`LogicalInput::exception`] is what says
/// which record's handler it enters.
///
/// Charges one `IrItems` per slot of every input it builds, before that input's locals are copied.
fn exception_inputs(
    method: &FrameMethod<'_>,
    canonical: &CanonicalCfg,
    block: &CanonicalBlock,
    throw_points: &[ThrowPoint],
    exit: &Frame,
    handler_ordinal: u32,
    budget: &mut Budget,
) -> Norm<Vec<ExceptionInput>> {
    let Some(row) = canonical
        .handler_rows
        .iter()
        .find(|row| row.ordinal == handler_ordinal)
    else {
        return inconsistent(format!(
            "block {:?} leaves through the exception edge of handler record {handler_ordinal}, \
             which no handler row of the graph states",
            block.id
        ));
    };
    let thrown = caught_reference(method, row)?;
    let mut inputs = Vec::new();
    for point in throw_points {
        if !point.handlers.contains(&handler_ordinal) {
            continue;
        }
        // The input is a second copy of the site's locals — plus the one slot its stack holds —
        // and every input of one edge is live at the same time as the snapshots they are taken
        // from: charged before the copy is made, not after it is handed over.
        charge_slots(budget, point.locals.len().saturating_add(1))?;
        inputs.push(ExceptionInput {
            bci: Some(point.bci),
            frame: Frame {
                locals: point.locals.clone(),
                stack: vec![thrown.clone()],
                touches: None,
            },
        });
    }
    if inputs.is_empty() {
        // The record protects this block and the block raises nothing: the handler is entered
        // from the block's own exit, whose locals are the last state the protected instructions
        // leave, with the caught reference as the whole stack.
        charge_slots(budget, exit.locals.len().saturating_add(1))?;
        inputs.push(ExceptionInput {
            bci: None,
            frame: Frame {
                locals: exit.locals.clone(),
                stack: vec![thrown],
                touches: None,
            },
        });
    }
    Ok(inputs)
}

/// The reference a handler is entered with: the record's catch type, or a conservative unknown
/// reference for a catch-all record.
pub(crate) fn caught_reference(method: &FrameMethod<'_>, row: &CanonicalHandlerRow) -> Norm<Value> {
    let Some(index) = row.catch_type_index else {
        // A catch-all record has no type: it catches anything, and the class file names nothing
        // this request could resolve to a type.
        return Ok(Value::Ref(RefType::Unknown));
    };
    let entry = jarde_reader::classfile::cp_entry(method.pool, index).map_err(|error| {
        Problem::Inconsistent(format!(
            "handler record {} catches the constant-pool entry {index}: {error}",
            row.ordinal
        ))
    })?;
    match &entry.kind {
        // The catch type's name is anchored to the request's loader exactly like every other
        // reference a descriptor names; the equality of two named references compares the loader
        // too, so a type from another loader never silently becomes this one.
        CpEntryKind::Class { name, .. } => Ok(named(name.0.clone(), method)),
        other => inconsistent(format!(
            "handler record {} catches the {} entry {index} where the format requires a class type",
            row.ordinal,
            cp_kind_name(other)
        )),
    }
}

/// The frame the method's own entry block is entered with: the parameters in the slots their
/// descriptor gives them, and `this` where the declaration puts it.
///
/// A constructor's local 0 is [`Value::UninitializedThis`] — the token the constructor call of
/// its own class or of its superclass converts — and every other
/// instance method's is an initialized reference to the declaring class. The descriptor and the
/// access flags decide both; nothing here guesses from the body.
fn entry_frame(method: &FrameMethod<'_>, facts: &MethodCodeFacts) -> Norm<Frame> {
    let slots = usize::from(facts.max_locals);
    let mut locals = vec![Value::Top; slots];
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
    }
    // One reading of the descriptor, and two facts from it: the slot class each parameter's value
    // has (this layer's vocabulary) and the slot each parameter starts at (the JVM layer's own
    // derivation, `this` included when the member is not `static`).
    let descriptor = descriptor_facts(method.descriptor, DescriptorKind::Method)
        .map_err(|error| Problem::Inconsistent(error.to_string()))?;
    let positions = parameter_positions(&descriptor, method.access_flags & ACC_STATIC != 0)
        .ok_or_else(|| {
            Problem::Inconsistent(format!(
                "the descriptor `{}` states more parameter slots than a local index holds",
                String::from_utf8_lossy(method.descriptor)
            ))
        })?;
    for (component, position) in descriptor.parameters().iter().zip(positions) {
        let next = usize::from(position);
        let (ty, name) = component_frame_type(method.descriptor, component)?;
        let width = ty.slots();
        if next + width > slots {
            return inconsistent(format!(
                "the descriptor `{}` needs more local slots than the method declares \
                 (max_locals {slots})",
                String::from_utf8_lossy(method.descriptor)
            ));
        }
        locals[next] = value_of(ty, name, method);
        if width == 2 {
            locals[next + 1] = Value::Second;
        }
    }
    Ok(Frame {
        locals,
        stack: Vec::new(),
        touches: None,
    })
}

/// What one run of the `frame` pass produced.
#[derive(Debug)]
pub(crate) enum FrameOutcome {
    /// The frames of every block the method's entry reaches.
    Frames(Box<FrameTable>),
    /// This build does not state the frames of this body: an uninitialized value stands where only
    /// an initialized reference is meaningful, and this pass neither converts it nor decides
    /// whether the bytes are legal. The message names the value and the instruction that needed
    /// them.
    Unsupported { message: String },
    /// The body contradicts itself; the message names the slot and the class file's own operand.
    Inconsistent { message: String },
}

/// One logical input of a block's entry state: where that state comes in from.
///
/// A logical input is finer than a canonical edge. One exception edge aggregates every throw site
/// its record covers, and each of those sites hands the handler its *own* state, so a consumer
/// that asks "how many values does this slot take here" must count these records and never the
/// aggregated edges — and an edge of a record that covers no throwing instruction of its source
/// hands over that source's exit as one input of its own. The source block names the normalization
/// context too, because a clone of a subroutine is a different node from the original and from
/// another clone.
///
/// One edge of a graph is one group of records, and the records of one group are the ones that
/// edge carried the last time its source ran. Two records of an exception table can name the same
/// handler for the same source — a named catch and a catch-all over one site is the shape — which
/// is why the group is not the source alone: the records of the two edges are two groups, and the
/// merge that built this state saw both of them.
///
/// The **count** is the invariant, not only the grouping: the records a block holds are exactly
/// the contributions its entry state was merged from, one per distinct edge of the graph that
/// feeds it. [`crate::frame::frames`] refuses a table whose two readings of that one fact
/// disagree, because a record list that folded two edges into one group would state a block
/// entered with a class none of the inputs it lists defines.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogicalInput {
    /// The block the state comes from.
    pub(crate) from: CanonicalBlockId,
    /// Ordinal of the exception-table record this input arrives through, for an input an
    /// exception edge carries into its handler; `None` for a plain transfer.
    ///
    /// This is the record the handler is entered by, and it is read off the edge itself. An input
    /// that a throw site hands over names that site in `throw_site` as well; an input the edge
    /// takes from its source's own exit — a record whose protected range covers no throwing
    /// instruction of that source — names none, and this field is the only statement of which
    /// record's handler it enters.
    pub(crate) exception: Option<u32>,
    /// BCI of the throwing instruction this input is taken at, for an input that arrives through
    /// an exception edge a throw site of the source feeds; `None` for a plain transfer, whose
    /// state is the source's exit, and for the exception edge of a record that covers no throwing
    /// instruction of the source, whose input is that same exit.
    pub(crate) throw_site: Option<u32>,
}

impl LogicalInput {
    /// The block this input's state comes from.
    pub fn from(&self) -> &CanonicalBlockId {
        &self.from
    }

    /// Ordinal of the exception-table record this input arrives through, or `None` for a plain
    /// transfer.
    pub fn exception(&self) -> Option<u32> {
        self.exception
    }

    /// BCI of the throwing instruction this input is taken at, or `None` for a plain transfer and
    /// for the exception edge of a record that covers no throwing instruction of its source.
    pub fn throw_site(&self) -> Option<u32> {
        self.throw_site
    }
}

/// The entry state of one canonical block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockFrame {
    /// Identity of the block this state belongs to.
    pub(crate) block: CanonicalBlockId,
    /// One entry per local slot.
    pub(crate) locals: Vec<Value>,
    /// The operand stack, bottom first.
    pub(crate) stack: Vec<Value>,
    /// One record per logical input this state was merged from, ordered by source block, then by
    /// the edge's exception-table ordinal — a plain transfer, whose ordinal is `None`, before the
    /// exception edges of that source — and then by throw site. Their number is the number of
    /// contributions the merge of this state really saw, which is what
    /// [`crate::frame::frames`] checks before it publishes the table.
    pub(crate) inputs: Vec<LogicalInput>,
}

impl BlockFrame {
    /// Identity of the block this entry state belongs to.
    pub fn block(&self) -> &CanonicalBlockId {
        &self.block
    }

    /// One entry per local slot, the entry state's locals array.
    pub fn locals(&self) -> &[Value] {
        &self.locals
    }

    /// The operand stack this block is entered with, bottom first.
    pub fn stack(&self) -> &[Value] {
        &self.stack
    }

    /// The logical inputs this entry state was merged from, in the pass's own order.
    pub fn inputs(&self) -> &[LogicalInput] {
        &self.inputs
    }
}

/// The published artifact: the entry state of every block the entry reaches, each with the
/// logical inputs it was merged from.
///
/// The table is derived storage whose consumers are the later slices of this crate and, through
/// the read-only handoff ([`crate::method_ir`]), the recovery layer above it: 3.5's graph and
/// 4.3's names are built over exactly these frames. What is published is the read surface of the
/// table — the entry states are read, never written, and the three fields stay private.
#[derive(Debug)]
pub struct FrameTable {
    /// One entry per reached block, in the canonical graph's block order.
    blocks: Vec<BlockFrame>,
    /// Slots of one locals array of this body (`max_locals`).
    locals_slots: usize,
    /// Deepest operand stack any entry state reaches, in slots.
    deepest_stack: usize,
}

impl FrameTable {
    /// The entry states, in the canonical graph's block order.
    pub fn blocks(&self) -> &[BlockFrame] {
        &self.blocks
    }

    /// The entry state of one block, or `None` when the entry cannot reach it.
    pub fn entry(&self, block: &CanonicalBlockId) -> Option<&BlockFrame> {
        self.blocks.iter().find(|entry| &entry.block == block)
    }

    /// Slots of one locals array of this body.
    pub fn locals_slots(&self) -> usize {
        self.locals_slots
    }

    /// Deepest operand stack any block is entered with.
    pub fn deepest_stack(&self) -> usize {
        self.deepest_stack
    }
}

/// Builds the frames of one canonical graph.
///
/// The entry state of each block is the **merge of its inputs**, computed by a worklist over the
/// canonical transfers until nothing changes. A plain transfer contributes its source's exit
/// state; an exception transfer contributes one state per throw site of its source that the
/// record covers, each with the locals of that site and a stack holding the single exception
/// reference. `Frames` covers every block the entry reaches, so a body whose decode stopped early
/// has the frames of its reliable prefix, exactly like the passes before this one. An unproven
/// state and a contradiction are two different outcomes, and neither of them is silently absorbed
/// into the other.
///
/// Charges, in order: one `IrItems` per slot of a block's entry state **before** that state is
/// stored (the locals array and the operand stack it is entered with), one `IrItems` per logical
/// input record before that record is stored, one more per merge that changes one, one
/// `AnalysisSteps` per worklist pop and per instruction examined, and one `IrItems` for the
/// published table. The operand stack an instruction builds *inside* a block is not derived
/// storage: it never leaves that block's transfer and the JVM's own slot ceiling bounds it. The
/// locals a throw site is entered with are the input of the states this pass stores, not storage
/// of their own — they are alive for one transfer and their merge is charged as a state.
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

/// Whether one frame entry starts a value a consumer may hold on its own.
///
/// `Top` is no readable value at all and has no value of its own, and the upper half of a
/// category-2 value belongs to the slot below it, so neither of them starts one: a consumer that
/// took either for a value would invent a definition no instruction produced.
pub(crate) fn starts_value(value: &Value) -> bool {
    !matches!(value, Value::Top | Value::Second)
}

/// The slots the method's entry block is entered with, as plain data.
///
/// The caller's own contribution is a participant of that block's entry phis like an incoming
/// edge is: a body that branches back to BCI 0 merges the caller's state with the incoming one,
/// and the incoming edge alone does not state the caller's half. `None` is the boundary
/// [`frames`] reports for the same state, which a run that published the frames of the entry
/// block cannot reach.
pub(crate) fn entry_slots(
    method: &FrameMethod<'_>,
    facts: &MethodCodeFacts,
) -> Result<Option<Vec<EntrySlot>>> {
    let state = match entry_frame(method, facts) {
        Ok(state) => state,
        Err(Problem::Budget(error)) => return Err(error),
        Err(Problem::Inconsistent(_) | Problem::Unproven(_)) => return Ok(None),
    };
    let mut slots = Vec::new();
    for (index, value) in state.locals.iter().enumerate() {
        if !starts_value(value) {
            continue;
        }
        slots.push(EntrySlot {
            region: SlotRegion::Local,
            slot: u32::try_from(index).unwrap_or(u32::MAX),
            value: value.clone(),
        });
    }
    for (depth, value) in state.stack.iter().enumerate() {
        if !starts_value(value) {
            continue;
        }
        slots.push(EntrySlot {
            region: SlotRegion::Stack,
            slot: u32::try_from(depth).unwrap_or(u32::MAX),
            value: value.clone(),
        });
    }
    Ok(Some(slots))
}

/// Replays one block over its **published** entry state and returns the slot accesses of each of
/// its instructions, in execution order.
///
/// The replay is the same transfer the fixpoint ran, on the state the table published for that
/// block: it exists so that a consumer reads the reads and writes of every instruction out of the
/// one dense table that produced the frames, instead of classifying the opcodes a second time —
/// the shapes of the `dup*`/`pop*`/`swap` family follow the values on the stack, so a
/// value-independent classification would be wrong for exactly those forms. A failure here is
/// therefore about this build's own artifact and not about the bytes: the frames of the block and
/// its replay disagree.
///
/// Charges: one `AnalysisSteps` per instruction examined, like the transfer it replays, one
/// `IrItems` per slot of the entry-state copy it replays over, and one `IrItems` per recorded
/// access before it is stored.
pub(crate) fn block_touches(
    facts: &MethodCodeFacts,
    block: &CanonicalBlock,
    entry: &BlockFrame,
    method: &FrameMethod<'_>,
    budget: &mut Budget,
) -> Result<TouchesOutcome> {
    match replay(facts, block, entry, method, budget) {
        Ok(accesses) => Ok(TouchesOutcome::Touches(accesses)),
        Err(Problem::Budget(error)) => Err(error),
        Err(Problem::Inconsistent(message)) => Ok(TouchesOutcome::Inconsistent { message }),
        Err(Problem::Unproven(message)) => Ok(TouchesOutcome::Unsupported { message }),
    }
}

/// The replay itself, as [`block_touches`] documents it.
fn replay(
    facts: &MethodCodeFacts,
    block: &CanonicalBlock,
    entry: &BlockFrame,
    method: &FrameMethod<'_>,
    budget: &mut Budget,
) -> Norm<Vec<InstructionTouches>> {
    // The replay mutates its own copy of the published entry state: slot storage like any other,
    // charged before the copy is made and not when the accesses it records are billed.
    charge_slots(budget, entry.locals.len().saturating_add(entry.stack.len()))?;
    let mut frame = Frame {
        locals: entry.locals.clone(),
        stack: entry.stack.clone(),
        touches: Some(Vec::new()),
    };
    let operands = facts.operands();
    let mut instructions = Vec::new();
    for index in instruction_indices(block, facts)? {
        let instruction = &facts.instructions[index];
        let operands = &operands[index];
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let row = TABLE[usize::from(operands.effective_opcode)];
        apply(
            method,
            facts,
            row,
            &block.id,
            instruction.bci,
            operands,
            &mut frame,
        )?;
        let touched = std::mem::take(
            frame
                .touches
                .as_mut()
                .expect("the replay is the one run that traces its own frame"),
        );
        budget.charge(
            CountedBudgetDimension::IrItems,
            u64::try_from(touched.len()).unwrap_or(u64::MAX),
        )?;
        instructions.push(InstructionTouches {
            bci: instruction.bci,
            opcode: operands.effective_opcode,
            stack_after: u32::try_from(frame.stack.len()).unwrap_or(u32::MAX),
            accesses: touched,
        });
    }
    Ok(instructions)
}

/// Identity of one canonical edge, as the logical input records are grouped by it.
///
/// One source block is **not** one edge: two records of an exception table can name the same
/// handler for the same source, and the two contributions they hand that handler are two. The
/// record's own ordinal is what tells the exception edges of one source apart; every edge that is
/// not an exception transfer carries `None`.
type EdgeKey = (CanonicalBlockId, Option<u32>);

/// Identity of one canonical edge as the *graph* states it: the node it leaves and its kind, which
/// carries the exception-table ordinal of an exception transfer.
///
/// It is not [`EdgeKey`], and the difference is the point: the two are derived in two different
/// places — this one from the canonical edge alone, `EdgeKey` from the edge the pass is walking —
/// so counting the contributions by this one is what makes "the records are the contributions the
/// merge saw" a statement about the graph instead of a restatement of the key.
type EdgeIdentity = (CanonicalBlockId, CanonicalEdgeKind);

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
    // The entry state is charged **before** it is allocated: the declared `max_locals` is exactly
    // what [`entry_frame`] allocates, because its operand stack starts empty.
    charge_slots(budget, usize::from(facts.max_locals))?;
    let first = entry_frame(method, facts)?;
    let mut entries: Vec<Option<Frame>> = vec![None; canonical.blocks.len()];
    let mut exits: Vec<Option<Frame>> = vec![None; canonical.blocks.len()];
    // The logical inputs of each block, keyed by the **edge** they arrive through: the inputs of
    // one edge are that edge's own contribution — its source's exit, or the throw-site states one
    // exception record hands over — so re-processing that edge replaces its records instead of
    // appending a second copy of them. The key is the source block plus the exception-table
    // ordinal of the edge, `None` for every edge that is not an exception transfer, because one
    // source and one handler are **not** one edge: two records of the table can name the same
    // handler for the same source, and a key that stopped at the source would let the second
    // edge's records overwrite the first's. The merge below has already seen both contributions
    // by then — it runs per contribution, not per edge — so losing one record here would publish
    // a state whose own class no input it lists defines.
    let mut inputs: Vec<BTreeMap<EdgeKey, Vec<LogicalInput>>> =
        vec![BTreeMap::new(); canonical.blocks.len()];
    // The contributions each block's entry state was **merged from**, keyed by the canonical edge
    // that carried them: one count per edge, set by that edge's last run, exactly like the record
    // group above it. Two numbers describe one target — how many records it holds and how many
    // contributions its merge saw — and they are equal for every block this pass publishes, which
    // is the invariant the publish step below refuses a table for breaking. The count is per
    // *edge* and not per group key on purpose: a key that folded two edges of the graph into one
    // group leaves the two numbers disagreeing instead of hiding the fold, and duplicate edges of
    // one (from, kind) — the same transfer stated twice by the canonization — are one entry, since
    // the merge of an identical contribution twice is that contribution.
    let mut merged: Vec<BTreeMap<EdgeIdentity, u64>> =
        vec![BTreeMap::new(); canonical.blocks.len()];
    entries[entry] = Some(first);
    let mut worklist = VecDeque::from([entry]);
    while let Some(position) = worklist.pop_front() {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        checkpoint(Phase::Transfer, budget)?;
        let block = canonical.blocks[position].clone();
        // The transfer mutates its own copy of the entry state, and that copy is a frame's worth
        // of slots of its own: it is what the exit state is derived in, so it is charged before
        // the clone like any other state this pass holds.
        let entry_state = {
            let before = entries[position]
                .as_ref()
                .expect("a block is queued only after it has an entry state");
            charge_frame(budget, before)?;
            before.clone()
        };
        let transfer = transfer_block(method, canonical, &block, facts, entry_state, budget)?;
        // The throw sites are as much a function of the entry state as the exit is, and the exit
        // is the weaker observation of the two: a block that overwrites every slot a throwing
        // instruction reads leaves the same exit on two different entries while the state its
        // handler is entered with has changed. An unchanged exit therefore only settles the
        // successors that read it — and those are exactly the successors this block's throw sites
        // do not feed. A block with no exception edge hands its sites to nobody, so nothing here
        // is left for a later visit to correct and the first transfer of it already did the work.
        let feeds_exception_edge = successors[position]
            .iter()
            .any(|(_, kind)| matches!(kind, CanonicalEdgeKind::Exception { .. }));
        if exits[position].as_ref() == Some(&transfer.exit) && !feeds_exception_edge {
            continue;
        }
        // The exit state is kept until this block runs again, to tell a second transfer of the
        // same entry state from a changed one: charged before the copy is made.
        charge_frame(budget, &transfer.exit)?;
        exits[position] = Some(transfer.exit.clone());
        for (target, kind) in &successors[position] {
            // One exception edge carries one input per throw site it aggregates; every other edge
            // carries the source's own exit state. Either way the states an edge hands over are
            // charged when they are allocated — here for the copy of the exit state, inside
            // [`exception_inputs`] for the per-site inputs — and not when a later merge happens
            // to keep them.
            let contributions: Vec<(Frame, Option<u32>)> = match kind {
                CanonicalEdgeKind::Exception { handler_ordinal } => exception_inputs(
                    method,
                    canonical,
                    &block,
                    &transfer.throw_points,
                    &transfer.exit,
                    *handler_ordinal,
                    budget,
                )?
                .into_iter()
                .map(|input| (input.frame, input.bci))
                .collect(),
                CanonicalEdgeKind::Normal
                | CanonicalEdgeKind::Call { .. }
                | CanonicalEdgeKind::Return { .. } => {
                    charge_frame(budget, &transfer.exit)?;
                    vec![(transfer.exit.clone(), None)]
                }
            };
            // The record an input arrives through is the edge's own ordinal, for every kind of
            // edge but a plain transfer: the handler is entered by that record, whether or not a
            // throw site of this block is what hands the state over.
            let exception = match kind {
                CanonicalEdgeKind::Exception { handler_ordinal } => Some(*handler_ordinal),
                CanonicalEdgeKind::Normal
                | CanonicalEdgeKind::Call { .. }
                | CanonicalEdgeKind::Return { .. } => None,
            };
            let target_id = &canonical.blocks[*target].id;
            let fed = u64::try_from(contributions.len()).unwrap_or(u64::MAX);
            let mut records = Vec::with_capacity(contributions.len());
            for (incoming, throw_site) in contributions {
                budget.charge(CountedBudgetDimension::IrItems, 1)?;
                records.push(LogicalInput {
                    from: block.id.clone(),
                    exception,
                    throw_site,
                });
                match entries[*target].as_mut() {
                    None => {
                        // The contribution was charged where it was allocated, and here its slots
                        // move into the table: the same slots are not billed a second time.
                        entries[*target] = Some(incoming);
                        worklist.push_back(*target);
                    }
                    Some(current) => {
                        let merged = merge_frame(budget, current, &incoming, target_id)?;
                        if merged != *current {
                            *current = merged;
                            worklist.push_back(*target);
                        }
                    }
                }
            }
            let key = (
                block.id.clone(),
                match kind {
                    CanonicalEdgeKind::Exception { handler_ordinal } => Some(*handler_ordinal),
                    CanonicalEdgeKind::Normal
                    | CanonicalEdgeKind::Call { .. }
                    | CanonicalEdgeKind::Return { .. } => None,
                },
            );
            inputs[*target].insert(key, records);
            // The same statement that records the group counts the contributions this edge handed
            // the merge: `fed` is the length of the very list the group was built from, so the two
            // numbers agree here and can only be made to disagree by a later run of this edge.
            merged[*target].insert((block.id.clone(), *kind), fed);
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

    // The invariant this pass owes the graph, and the reason it is checked **here** rather than
    // left to the pass behind: the logical input records of a block are the contributions its
    // entry state was merged from. Both are built in one statement of the walk above — the records
    // the merge was handed, and the count of them under the edge that carried them — so a block
    // whose two numbers disagree holds a record list that folded some of the graph's edges into
    // one group. That is a defect of this pass's own bookkeeping and not a fact about the method,
    // and it has to be refused as such: a folded list states a block entered with a class no input
    // it lists defines, and the phase behind would report the *bytes* as self-contradictory for it
    // (`ir_ssa_inconsistent`, on a legal body). Refusing keeps the defect where it was made.
    //
    // The check is a named function of its own so the refusal can be driven with the two readings
    // of one fact directly (see the unit test), instead of only through a body that folds them.
    for (position, (records, contributions)) in inputs.iter().zip(merged.iter()).enumerate() {
        records_are_the_contributions(&canonical.blocks[position].id, records, contributions)?;
    }

    let deepest_stack = entries
        .iter()
        .flatten()
        .map(|frame| frame.stack.len())
        .max()
        .unwrap_or(0);
    let mut blocks = Vec::new();
    for ((position, state), records) in entries.into_iter().enumerate().zip(inputs) {
        let Some(frame) = state else {
            continue;
        };
        // The block's own record and the block id it names. The entry state itself **moves** out
        // of the table here — its slots were charged where they were derived — and so do the
        // logical inputs, which were charged when they were recorded.
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        blocks.push(BlockFrame {
            block: canonical.blocks[position].id.clone(),
            locals: frame.locals,
            stack: frame.stack,
            inputs: records.into_values().flatten().collect(),
        });
    }
    budget.charge(CountedBudgetDimension::IrItems, 1)?;
    Ok(FrameTable {
        blocks,
        locals_slots: usize::from(facts.max_locals),
        deepest_stack,
    })
}

/// The invariant of one block's published record list: **the records it holds are the
/// contributions its entry state was merged from**.
///
/// The two numbers are two readings of one fact — the walk above writes the group and the count of
/// the very same contributions in one statement — so a block whose record list folded two edges of
/// the graph into one group is a block whose list states fewer inputs than its state was built
/// from. Such a state is entered with a class none of the inputs it lists defines: the merge took
/// both edges' contributions, the list kept only one of their records, and the phase behind reads
/// the disagreement as a contradiction of the *bytes*. It is not one, so the record list is refused
/// here, where the two halves of the fact still sit next to each other.
///
/// `contributions` is keyed by the **edge the graph states** ([`EdgeIdentity`]) rather than by the
/// group key the list is keyed with, which is what gives this check teeth: counting the
/// contributions under the same key the records are grouped by would restate the grouping instead
/// of comparing it with the graph. Duplicate edges of one `(from, kind)` are one entry on purpose —
/// a transfer the canonization states twice hands the merge the same contribution twice, and the
/// merge of a contribution with itself is that contribution.
///
/// A refusal is [`Problem::Inconsistent`], the module's own code for two artifacts of one run that
/// must agree and do not; it is the same path the other graph defects of this pass are refused
/// through (an edge naming a block the graph does not hold, an exception edge whose record no row
/// states).
fn records_are_the_contributions(
    block: &CanonicalBlockId,
    records: &BTreeMap<EdgeKey, Vec<LogicalInput>>,
    contributions: &BTreeMap<EdgeIdentity, u64>,
) -> Norm<()> {
    let held: u64 = records
        .values()
        .map(|group| u64::try_from(group.len()).unwrap_or(u64::MAX))
        .sum();
    let merged_from: u64 = contributions.values().copied().sum();
    if held != merged_from {
        return inconsistent(format!(
            "block {block:?} holds {held} logical input record(s) for the {} edge(s) that feed it, \
             while its entry state was merged from {merged_from} contribution(s): the records and \
             the merge are two readings of one fact and cannot disagree",
            contributions.len()
        ));
    }
    Ok(())
}

/// Charges a run of derived frame slots, **before** the storage they count is allocated.
///
/// The budget's own contract for [`CountedBudgetDimension::IrItems`] is one item per frame or
/// local slot, charged before the allocation. This pipeline holds more of those at once than the
/// table it finally publishes: the worklist's working copy of an entry state, the exit state kept
/// to tell a second transfer from a changed one, the per-site snapshots a transfer holds, and the
/// states one edge hands to its successors. Every one of them is slot storage that exists while
/// the run is bounded by a limit, so every one of them is charged here in the statement before it
/// is allocated, and nothing in this module is exempt for being short-lived.
fn charge_slots(budget: &mut Budget, slots: usize) -> Norm<()> {
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(slots).unwrap_or(u64::MAX),
    )?;
    Ok(())
}

/// Charges one derived frame state the slots it holds, before it is allocated.
///
/// The state the charge is taken for is the one the caller is about to allocate, which is why this
/// takes the *shape* to charge for and not a state that already exists: a merged frame has exactly
/// the shape of the state it merges into, and a copy has exactly the shape of its source.
fn charge_frame(budget: &mut Budget, frame: &Frame) -> Norm<()> {
    charge_slots(budget, frame.locals.len().saturating_add(frame.stack.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call_context::{CallContextOutcome, call_contexts};
    use crate::cfg::raw_cfg;
    use jarde_reader::budget::{BudgetDimension, Limits, UsageSnapshot};
    use jarde_reader::classfile::{
        BytecodeStop, ExceptionHandlerFact, InstructionFact, LocalDebugTable, LocalOperand,
        class_facts, method_code_facts,
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
            super_class: Some(b"java/lang/Object"),
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
            // An assembled body states no debug table: the names come from a class file's own
            // `LocalVariableTable`, which no assembled fixture has (P3 3.1).
            LocalDebugTable::Absent,
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
            super_class: Some(b"java/lang/Object"),
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
            super_class: Some(b"java/lang/Object"),
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
            super_class: Some(b"java/lang/Object"),
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

    /// A token may only be moved: consuming one as an initialized reference stops the body under
    /// this build's boundary code, and never as a contradiction of its bytes.
    ///
    /// That stop outlives the 4.2 conversions. What it reports is no longer "the conversion is
    /// missing" but "no conversion is defined for this use, and whether such a body is legal at
    /// all is the verifier's question, which this pass does not answer".
    #[test]
    fn consuming_an_uninitialized_value_stops_the_body() {
        // 0  new Test (constant pool 2)   stack <- uninitialized(0)
        // 3  ifnull 6                     consumes it as a reference, which it is not yet
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
            message.contains("uninitialized") && message.contains("moved, or converted"),
            "the stop names the state and why this instruction may not take it: {message}"
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

    // -- Initialization: the `new`/`<init>` chain and the uninitialized `this` -----------------

    /// One `Methodref` naming a constructor of the caller's class: the entry an
    /// `invokespecial <init>` names, whose class the conversions judge their target by.
    fn constructor_ref(index: u16, owner: &[u8], descriptor: &[u8]) -> CpEntryFacts {
        CpEntryFacts {
            index,
            span: ByteSpan::new(CODE_OFFSET, 0),
            kind: CpEntryKind::MethodRef {
                class_index: 0,
                name_and_type_index: 0,
                owner: JvmBytes(owner.to_vec()),
                name: JvmBytes(b"<init>".to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
        }
    }

    /// One `Fieldref` of a synthetic pool: the entry whose **declaring class** the restricted
    /// `putfield` of JVMS 4.10.1.9 reads beside its descriptor.
    fn field_ref(index: u16, owner: &[u8], name: &[u8], descriptor: &[u8]) -> CpEntryFacts {
        CpEntryFacts {
            index,
            span: ByteSpan::new(CODE_OFFSET, 0),
            kind: CpEntryKind::FieldRef {
                class_index: 0,
                name_and_type_index: 0,
                owner: JvmBytes(owner.to_vec()),
                name: JvmBytes(name.to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
        }
    }

    /// The initialized reference a conversion produces for one class name, anchored to the loader
    /// every fixture of this module uses.
    fn initialized(name: &[u8]) -> Value {
        Value::Ref(RefType::Named {
            name: name.to_vec(),
            loader: Box::new(LoaderId("app".to_string())),
        })
    }

    /// The token of one value, or the panic that says it is not an uninitialized one.
    fn token_of(value: &Value) -> &NewSite {
        match value {
            Value::Uninitialized { new_site } => new_site,
            other => panic!("the slot must hold an uninitialized token, got {other:?}"),
        }
    }

    /// One run of the pass over a **decoded** fixture body as the constructor of `Test`, whose
    /// class file names `java/lang/Object` as its superclass.
    fn constructor_of(fixture: &Fixture, descriptor: &[u8]) -> FrameOutcome {
        let loader = LoaderId("app".to_string());
        let method = FrameMethod {
            access_flags: 0,
            name: b"<init>",
            descriptor,
            owner: b"Test",
            super_class: Some(b"java/lang/Object"),
            pool: &fixture.pool,
            loader: &loader,
        };
        frames(&fixture.facts, &fixture.canonical, &method, &mut budget())
            .expect("a legal run is answered")
    }

    /// The same for one synthetic body.
    fn constructor_frames(synthetic: &Synthetic, descriptor: &[u8]) -> FrameOutcome {
        let loader = LoaderId("app".to_string());
        let method = FrameMethod {
            access_flags: 0,
            name: b"<init>",
            descriptor,
            owner: b"Test",
            super_class: Some(b"java/lang/Object"),
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

    /// The applicable `invokespecial <init>` converts **every alias** of the token it was given —
    /// the locals and the operand stack alike — into an initialized reference, and it converts
    /// only that token's aliases.
    #[test]
    fn a_constructor_call_initializes_every_alias_of_one_new_site() {
        // 0  new Test (#2)                 stack <- the token of site 0
        // 3  dup                           one alias into local 1
        // 4  astore_1
        // 5  dup                           one alias into local 2
        // 6  astore_2
        // 7  dup                           one alias stays on the stack
        // 8  invokespecial Test.<init>()V (#18)  -- the applicable call
        // 11 (the diamond's condition; both arms reach the join unchanged)
        let (mut code, join) = diamond(
            &[
                0xbb, 0x00, 0x02, // new Test
                0x59, 0x4c, // dup, astore_1
                0x59, 0x4d, // dup, astore_2
                0x59, // dup
                0xb7, 0x00, 0x12, // invokespecial #18
            ],
            &[0x03, 0x57], // iconst_0, pop
            &[0x03, 0x57],
        );
        code.push(0xb1); // return
        let fixture = fixture_body(&code, 4);
        let table = frames_or_panic(frames_of(&fixture));
        let entry = entry_of(&table, join);
        assert_eq!(
            entry.locals[1],
            initialized(b"Test"),
            "the alias in local 1 is converted, not only the receiver that was consumed"
        );
        assert_eq!(
            entry.locals[2],
            initialized(b"Test"),
            "and the alias in local 2 as well"
        );
        assert_eq!(
            entry.stack,
            vec![initialized(b"Test")],
            "and the alias the call did not consume, which is still on the stack"
        );
    }

    /// The conversion is one comparison against one token, and that is what makes "every alias"
    /// exact: it reaches the locals and the operand stack alike, and it leaves every other value
    /// as it stands — the token of another `new` included, however equal the two values' shapes
    /// are.
    #[test]
    fn the_conversion_reaches_every_alias_and_no_other_value() {
        let site = |bci| NewSite {
            block: CanonicalBlockId {
                bci: 0,
                path: Vec::new(),
            },
            bci,
        };
        let token = Value::Uninitialized { new_site: site(0) };
        let other = Value::Uninitialized { new_site: site(3) };
        let mut frame = Frame {
            locals: vec![token.clone(), other.clone(), Value::Top],
            stack: vec![token.clone(), other.clone()],
            touches: None,
        };
        frame.convert_token(&token, &Value::Int);
        assert_eq!(
            frame.locals,
            vec![Value::Int, other.clone(), Value::Top],
            "the alias in local 0 is converted and local 1's own token is not"
        );
        assert_eq!(
            frame.stack,
            vec![Value::Int, other],
            "the stack is read the same way"
        );
        assert!(
            !frame
                .locals
                .iter()
                .chain(frame.stack.iter())
                .any(|slot| slot == &token),
            "every alias of the token is converted: no slot of either plane still holds it"
        );
    }

    /// Every move of the `dup`/`pop`/`swap` family and both local accesses carry a token without
    /// interpreting it: `swap` exchanges it with a basic value, `pop` takes it, and `aload`/`astore`
    /// carry it — and none of them converts it, because a token is converted by the constructor
    /// call that constructs it and by nothing else.
    #[test]
    fn the_moves_carry_a_token_without_converting_it() {
        // 0  new Test (#2)       stack <- the token of site 0
        // 3  dup                 one alias into local 1
        // 4  astore_1
        // 5  iconst_0            a basic value to exchange with
        // 6  swap                the token moves past it
        // 7  pop                 ... and is taken as a value
        // 8  pop
        // 9  aload_1             the alias again, into local 2
        // 10 astore_2
        let (mut code, join) = diamond(
            &[
                0xbb, 0x00, 0x02, // new Test
                0x59, 0x4c, // dup, astore_1
                0x03, 0x5f, // iconst_0, swap
                0x57, 0x57, // pop, pop
                0x2b, 0x4d, // aload_1, astore_2
            ],
            &[0x03, 0x57], // iconst_0, pop
            &[0x03, 0x57],
        );
        code.push(0xb1); // return
        let fixture = fixture_body(&code, 3);
        let table = frames_or_panic(frames_of(&fixture));
        let entry = entry_of(&table, join);
        let site = NewSite {
            block: CanonicalBlockId {
                bci: 0,
                path: Vec::new(),
            },
            bci: 0,
        };
        assert_eq!(
            token_of(&entry.locals[1]),
            &site,
            "the alias copied into local 1 is the token of the `new`, not a reference"
        );
        assert_eq!(
            token_of(&entry.locals[2]),
            &site,
            "and the one `aload`/`astore` carried into local 2 is the same token"
        );
    }

    /// Two `new`s of one class are two tokens: the constructor call of one converts its own
    /// aliases and leaves the other value uninitialized.
    #[test]
    fn two_new_sites_are_two_tokens_one_call_does_not_convert() {
        // 0  new Test (#2)                 site 0 -> local 1
        // 3  astore_1
        // 4  new Test (#2)                 site 4 -> local 2
        // 7  astore_2
        // 8  aload_1
        // 9  invokespecial Test.<init>()V (#18)  -- constructs the value of local 1 only
        let (mut code, join) = diamond(
            &[
                0xbb, 0x00, 0x02, // new Test
                0x4c, // astore_1
                0xbb, 0x00, 0x02, // new Test
                0x4d, // astore_2
                0x2b, // aload_1
                0xb7, 0x00, 0x12, // invokespecial #18
            ],
            &[0x03, 0x57],
            &[0x03, 0x57],
        );
        code.push(0xb1);
        let fixture = fixture_body(&code, 4);
        let table = frames_or_panic(frames_of(&fixture));
        let entry = entry_of(&table, join);
        assert_eq!(
            entry.locals[1],
            initialized(b"Test"),
            "the site the call was given is initialized"
        );
        assert_eq!(
            token_of(&entry.locals[2]).bci,
            4,
            "the other site is not converted by a call that never saw its token"
        );
        assert!(
            entry.locals[2].is_uninitialized(),
            "and it is still an uninitialized value: {:?}",
            entry.locals[2]
        );
    }

    /// Two paths that hold **different** tokens in one local reach their join with an unusable
    /// slot rather than with one of the tokens: a token belongs to one `new` site and to no other,
    /// so the local merge answers `Top` — the rule local merging always had — and a later read of
    /// that slot is where the failure belongs.
    #[test]
    fn two_new_sites_of_two_paths_do_not_merge_into_one_token() {
        let (mut code, join) = diamond(
            &[],
            &[0xbb, 0x00, 0x02, 0x4d], // new Test; astore_2
            &[0xbb, 0x00, 0x02, 0x4d], // new Test; astore_2
        );
        code.push(0xb1);
        let fixture = fixture_body(&code, 4);
        let table = frames_or_panic(frames_of(&fixture));
        assert_eq!(
            entry_of(&table, join).locals[2],
            Value::Top,
            "the two sites are two tokens, and the slot answers neither of them"
        );
    }

    /// A constructor's `this` is converted by an `<init>` of its own class or of its
    /// `super_class` — the two calls JVMS 4.9.2 allows it — and the reference it becomes is the
    /// class being constructed, whichever of the two calls did it.
    #[test]
    fn a_constructor_call_reaches_the_own_class_or_the_superclass() {
        // 0  aload_0                        the uninitialized `this`
        // 1  invokespecial Test.<init>()V (#18)      -- the class's own constructor
        // 4  aload_0 / astore_1             the same `this`, now a reference
        let (mut code, join) = diamond(
            &[
                0x2a, // aload_0
                0xb7, 0x00, 0x12, // invokespecial #18
                0x2a, 0x4c, // aload_0, astore_1
            ],
            &[0x03, 0x57],
            &[0x03, 0x57],
        );
        code.push(0xb1);
        let fixture = fixture_body(&code, 4);
        let table = frames_or_panic(constructor_of(&fixture, b"()V"));
        let entry = entry_of(&table, join);
        assert_eq!(
            entry.locals[0],
            initialized(b"Test"),
            "the receiver's own slot is converted by its class's `<init>`"
        );
        assert_eq!(
            entry.locals[1],
            initialized(b"Test"),
            "and a load after the call pushes the initialized reference"
        );

        // The superclass's constructor is the other applicable call, and the class it converts
        // `this` into is still the class being constructed.
        let (mut code, join) = diamond(
            &[
                0x2a, // aload_0
                0xb7, 0x00, 0x13, // invokespecial #19: java/lang/Object.<init>()V
                0x2a, 0x4c, // aload_0, astore_1
            ],
            &[0x03, 0x57],
            &[0x03, 0x57],
        );
        code.push(0xb1);
        let fixture = fixture_body(&code, 4);
        let table = frames_or_panic(constructor_of(&fixture, b"()V"));
        let entry = entry_of(&table, join);
        assert_eq!(
            entry.locals[0],
            initialized(b"Test"),
            "`super.<init>` converts the `this` of the class being constructed, not of the \
             superclass whose constructor ran"
        );
        assert_eq!(entry.locals[1], initialized(b"Test"));
    }

    /// The normal successor of a constructor call is converted and its **exception** successor is
    /// not: the state a handler is entered with is the state at the call, taken before the call
    /// took effect. The two successors of one instruction do not share a state.
    #[test]
    fn a_constructor_call_leaves_its_exception_input_unconverted() {
        // A constructor of `Test` calling `super.<init>`: the block below performs the call and
        // the record protects exactly that block, so both successors of the call are entered from
        // it — the normal one through the diamond, the handler through the exception edge.
        //
        // 0    aload_0                       the uninitialized `this`
        // 1..3 invokespecial java/lang/Object.<init>()V (#19)
        // 4    iconst_0 / 5..6 ifeq 12
        // 8    nop                           the then arm
        // 9..11 goto 13
        // 12   nop                           the else arm
        // 13   return                        the join: two predecessors, so not fused away
        // 14   pop / 15 return               the handler entry
        let (mut code, join) = diamond(&[0x2a, 0xb7, 0x00, 0x13], &[0x00], &[0x00]);
        code.push(0xb1); // return
        code.push(0x57); // pop
        code.push(0xb1); // return
        let fixture = fixture_body(&code, 4);
        let synthetic = with_handlers(
            fixture.facts,
            fixture.pool,
            vec![ExceptionHandlerFact {
                ordinal: 0,
                start_bci: 0,
                end_bci: 8,
                handler_bci: 14,
                catch_type_index: None,
            }],
        );
        let table = frames_or_panic(constructor_frames(&synthetic, b"()V"));
        let normal = entry_of(&table, join);
        let handler = entry_of(&table, 14);
        assert_eq!(
            normal.locals[0],
            initialized(b"Test"),
            "the normal successor is entered with the converted `this`"
        );
        assert_eq!(
            handler.locals[0],
            Value::UninitializedThis,
            "the handler is entered with the state at the call, not with the state the call \
             completed into"
        );
        assert_eq!(
            handler.stack,
            vec![Value::Ref(RefType::Unknown)],
            "its stack holds the single exception reference, as every handler entry does"
        );
        assert_ne!(
            handler.locals[0], normal.locals[0],
            "one instruction's two successors never share a state"
        );
    }

    /// A constructor call that is not applicable to the token it was given stops the body under
    /// the boundary code. The pass does not convert it "because an `<init>` was called", and it
    /// does not report it as a contradiction either: whether such a body is legal is the
    /// verifier's question, and the report's `verification` plane stays `NotPerformed`.
    #[test]
    fn a_constructor_call_that_is_not_applicable_stops_the_body() {
        // 0  aload_0
        // 1  invokespecial Other.<init>()V (#1): neither the own class nor the superclass
        let synthetic = synthetic_body(
            vec![local(0, 0x2a, 0), call(1, 0xb7, 1), plain(4, 0xb1)],
            vec![constructor_ref(1, b"Other", b"()V")],
            1,
            5,
        );
        let message = unproven(constructor_frames(&synthetic, b"()V"));
        assert!(
            message.contains("Other") && message.contains("4.9.2"),
            "the stop names the class the call names and the rule it is read against: {message}"
        );

        // A value a `new` made is constructed by its own class's `<init>` and by no other's.
        let synthetic = synthetic_body(
            vec![
                call(0, 0xbb, 1), // new Test (#1)
                plain(3, 0x59),   // dup
                call(4, 0xb7, 2), // invokespecial Other.<init>()V (#2)
                plain(7, 0xb1),
            ],
            vec![class_ref(1, b"Test"), constructor_ref(2, b"Other", b"()V")],
            2,
            8,
        );
        let message = unproven(synthetic_frames(&synthetic, b"()V"));
        assert!(
            message.contains("Other") && message.contains("Test"),
            "the stop names both the class the call names and the one the `new` allocated: {message}"
        );

        // An invocation that is not an `<init>` at all is not a constructor call, whoever the
        // receiver would be.
        let synthetic = synthetic_body(
            vec![
                call(0, 0xbb, 1), // new Test (#1)
                plain(3, 0x59),   // dup
                call(4, 0xb6, 2), // invokevirtual Test.m()V (#2)
                plain(7, 0xb1),
            ],
            vec![class_ref(1, b"Test"), method_ref(2, b"()V")],
            2,
            8,
        );
        let message = unproven(synthetic_frames(&synthetic, b"()V"));
        assert!(
            message.contains("<init>") && message.contains("does not name"),
            "the stop says the call is not the constructor call: {message}"
        );

        // And an `<init>` call on a receiver that is already initialized: the pass has one
        // transition for a constructor call, the conversion of a token, and no other — so it stops
        // rather than reading an ordinary `void` call into bytes JVMS 4.9.2 forbids.
        let synthetic = synthetic_body(
            vec![
                call(0, 0xbb, 1), // new Test (#1)
                plain(3, 0x59),   // dup
                call(4, 0xb7, 2), // invokespecial Test.<init>()V (#2): the token is converted
                call(7, 0xb7, 2), // invokespecial Test.<init>()V (#2) again, on the reference
                plain(10, 0xb1),
            ],
            vec![class_ref(1, b"Test"), constructor_ref(2, b"Test", b"()V")],
            1,
            11,
        );
        let message = unproven(synthetic_frames(&synthetic, b"()V"));
        assert!(
            message.contains("already") || message.contains("not an uninitialized value"),
            "the stop names the receiver's state: {message}"
        );
    }

    /// An invocation of a method called `<init>` under an opcode other than `invokespecial` stops
    /// the body under the boundary code, exactly like an `invokespecial <init>` this pass cannot
    /// convert.
    ///
    /// The name is the entry's own fact, so `invokevirtual` and `invokestatic` of a method called
    /// `<init>` name a constructor call just as `invokespecial` does — and the one transition this
    /// pass has for a call named `<init>` belongs to the `invokespecial` that constructs a token.
    /// The other opcodes construct nothing, so the body stops instead of the entry being read as an
    /// ordinary `void` call, and the stop is the boundary's own code rather than a contradiction:
    /// whether such bytes are legal is the verifier's question.
    #[test]
    fn an_init_named_invocation_that_is_not_an_invokespecial_stops_the_body() {
        // 0  aconst_null  the receiver such a call would need
        // 1  invokevirtual Test.<init>()V (#1)
        // 4  return
        let synthetic = synthetic_body(
            vec![plain(0, 0x01), call(1, 0xb6, 1), plain(4, 0xb1)],
            vec![constructor_ref(1, b"Test", b"()V")],
            1,
            5,
        );
        let message = unproven(synthetic_frames(&synthetic, b"()V"));
        assert!(
            message.contains("0xb6")
                && message.contains("Test")
                && message.contains("not an `invokespecial`"),
            "the stop names the opcode, the class the entry names and what is missing: {message}"
        );

        // The same entry under `invokestatic`, which names no receiver at all.
        // 0  invokestatic Test.<init>()V (#1)
        // 3  return
        let synthetic = synthetic_body(
            vec![call(0, 0xb8, 1), plain(3, 0xb1)],
            vec![constructor_ref(1, b"Test", b"()V")],
            1,
            4,
        );
        let message = unproven(synthetic_frames(&synthetic, b"()V"));
        assert!(
            message.contains("0xb8")
                && message.contains("Test")
                && message.contains("not an `invokespecial`"),
            "the stop names the opcode, the class the entry names and what is missing: {message}"
        );
    }

    /// JVMS 4.10.1.9's second use of an uninitialized `this`: an instance initialization method
    /// may assign a field through the `this` it has not initialized yet. The restricted form is
    /// read from the **name** the `putfield`'s `Fieldref` gives for the field's owner and from the
    /// class the method is declared in — never from a field table, which this layer does not hold —
    /// and every other shape of the same instruction stops. The accept case below is that boundary
    /// in its sharpest form: the fixture's pool holds the `Fieldref` and its class declares no
    /// field at all, and the instruction is accepted because the entry names the own class.
    #[test]
    fn a_constructor_may_store_its_own_field_through_the_uninitialized_this() {
        // 0  aload_0        the uninitialized `this`
        // 1  iconst_0       the value
        // 2  putfield #1
        // 5  return
        let own_field = vec![
            local(0, 0x2a, 0),
            plain(1, 0x03),
            call(2, 0xb5, 1),
            plain(5, 0xb1),
        ];
        let synthetic = synthetic_body(
            own_field.clone(),
            vec![field_ref(1, b"Test", b"x", b"I")],
            1,
            6,
        );
        assert!(
            matches!(
                constructor_frames(&synthetic, b"()V"),
                FrameOutcome::Frames(_)
            ),
            "a `Fieldref` naming the class being constructed is the one the rule reaches"
        );

        // The same instruction on another class's `Fieldref` stops: the rule is decided by that
        // name, not by `putfield` in general.
        let synthetic = synthetic_body(own_field, vec![field_ref(1, b"Other", b"x", b"I")], 1, 6);
        let message = unproven(constructor_frames(&synthetic, b"()V"));
        assert!(
            message.contains("uninitialized `this`") && message.contains("4.10.1.9"),
            "the stop names the state and the rule that does not reach here: {message}"
        );

        // A value a `new` made has no `putfield` in its grammar at all: the rule is `this`'s.
        let synthetic = synthetic_body(
            vec![
                call(0, 0xbb, 1), // new Test (#1)
                plain(3, 0x59),   // dup
                plain(4, 0x03),   // iconst_0
                call(5, 0xb5, 2), // putfield Test.x:I (#2)
                plain(8, 0xb1),
            ],
            vec![class_ref(1, b"Test"), field_ref(2, b"Test", b"x", b"I")],
            2,
            9,
        );
        let message = unproven(synthetic_frames(&synthetic, b"()V"));
        assert!(
            message.contains("target of a field access") && message.contains("4.10.1.9"),
            "the stop says which value may not stand there and why: {message}"
        );
    }

    /// A block entered through an exception edge is entered with the state its throw site hands
    /// it: the locals of that site and the single exception reference on the stack — never a state
    /// invented from the aggregated edge.
    ///
    /// Until the handler entries landed this body stopped under `ir_frame_deferred` with a message
    /// naming the edge. The entry state it gets now is the stronger claim, so the assertion states
    /// the state instead of the stop.
    #[test]
    fn an_exception_edge_is_entered_with_the_state_of_its_throw_site() {
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
        let synthetic = with_handlers(
            synthetic.facts,
            synthetic.pool,
            vec![ExceptionHandlerFact {
                ordinal: 0,
                start_bci: 0,
                end_bci: 3,
                handler_bci: 4,
                catch_type_index: None,
            }],
        );
        let table = frames_or_panic(synthetic_frames(&synthetic, b"()V"));
        let handler = entry_of(&table, 4);
        assert_eq!(
            handler.stack,
            vec![Value::Ref(RefType::Unknown)],
            "a catch-all record names no type, so the entry holds a conservative unknown reference"
        );
        assert_eq!(
            handler.locals,
            vec![Value::Top; 4],
            "the handler is entered with the locals of the throwing instruction itself"
        );
        assert_eq!(
            handler.inputs,
            vec![LogicalInput {
                from: CanonicalBlockId {
                    bci: 0,
                    path: Vec::new(),
                },
                exception: Some(0),
                throw_site: Some(2),
            }],
            "the single site of the block is the single logical input of the handler"
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
            // A body that stopped before the nested attributes were walked states no name.
            LocalDebugTable::Unstated,
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

    /// The pass bills the two dimensions its row declares and no other, and what it stores is what
    /// the item charge counts: every run of derived frame slots it allocates — the entry state, the
    /// working copy of each visit, the exit state a transfer keeps, the state an edge hands over,
    /// the state a merge builds — plus one item per logical input record, per published block and
    /// for the table itself.
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
        let records = table
            .blocks()
            .iter()
            .map(|block| u64::try_from(block.inputs.len()).expect("a small fixture"))
            .sum::<u64>();
        // Four entered blocks of four local slots, the one merge that changed the join's state, the
        // four logical inputs of this diamond's four edges, and the published table.
        assert_eq!(entered, 4);
        assert_eq!(locals, 4);
        assert_eq!(records, 4);
        // The run visits five blocks: the entry block, both arms, the join, and the join a second
        // time, because the second arm's state changed the join's entry and queued it again. One of
        // those five transfers finds the exit it already had and stops there, so four of them keep
        // an exit state. Every one of these is an allocation of `locals` slots and every one is
        // billed, which is what this assertion is about.
        let visits = 5;
        let transfers = 4;
        let merges = 1;
        assert_eq!(
            usage.ir_items,
            locals                      // the entry state
                + visits * locals       // the working copy each visit derives its exit in
                + transfers * locals    // the exit state each of those transfers keeps
                + records * locals      // the state each edge hands to its successor
                + records               // one item per logical input record
                + merges * locals       // the state the one changing merge built
                + entered               // one item per published block record
                + 1 // the published table itself
        );
        assert_eq!(usage.ir_edges, 0, "this pass builds no edge of its own");
        assert!(usage.analysis_steps > 0, "the worklist ran");
        assert_eq!(
            table.deepest_stack(),
            0,
            "every block is entered empty here"
        );
    }

    // -- Handler entries, throw sites and the identity of a `new` -------------------------

    /// One body with the exception records the caller states, and the canonical graph derived from
    /// those same facts again.
    ///
    /// The builder's class files carry no exception table, so the records are added here as the
    /// facts the reader would have published: the frames pass reads them off the facts, and the raw
    /// graph — and therefore the canonical graph — is built from them too.
    fn with_handlers(
        facts: MethodCodeFacts,
        pool: Vec<CpEntryFacts>,
        handlers: Vec<ExceptionHandlerFact>,
    ) -> Synthetic {
        let mut facts = facts;
        facts.exception_handler_count = u32::try_from(handlers.len()).expect("a small fixture");
        facts.exception_handlers = handlers;
        let canonical = canonical_of(&facts, 52);
        Synthetic {
            facts,
            pool,
            canonical,
        }
    }

    /// A body of the `jsr` era: only a class file of major ≤ 50 may hold a `jsr`/`ret`, and the
    /// call-context walk reads the version to decide that.
    fn jsr_body(
        code: Vec<(InstructionFact, InstructionOperands)>,
        pool: Vec<CpEntryFacts>,
        max_locals: u16,
        code_length: u32,
    ) -> Synthetic {
        let facts = MethodCodeFacts::from_parts(
            50,
            max_locals,
            ByteSpan::new(CODE_OFFSET, u64::from(code_length)),
            code,
            Vec::new(),
            0,
            ExecutionReport::Complete {
                usage: UsageSnapshot::default(),
            },
            None,
            // An assembled body states no debug table: the names come from a class file's own
            // `LocalVariableTable`, which no assembled fixture has (P3 3.1).
            LocalDebugTable::Absent,
        );
        let canonical = canonical_of(&facts, 50);
        Synthetic {
            facts,
            pool,
            canonical,
        }
    }

    /// One `Class` entry of a synthetic pool: the kind a `new` names and a handler record catches.
    fn class_ref(index: u16, name: &[u8]) -> CpEntryFacts {
        CpEntryFacts {
            index,
            span: ByteSpan::new(CODE_OFFSET, 0),
            kind: CpEntryKind::Class {
                name_index: 0,
                name: JvmBytes(name.to_vec()),
            },
        }
    }

    /// One `Methodref` of a synthetic pool under the name the caller spells — `<init>`, which
    /// [`method_ref`] does not write.
    fn method_ref_named(index: u16, name: &[u8], descriptor: &[u8]) -> CpEntryFacts {
        CpEntryFacts {
            index,
            span: ByteSpan::new(CODE_OFFSET, 0),
            kind: CpEntryKind::MethodRef {
                class_index: 0,
                name_and_type_index: 0,
                owner: JvmBytes(b"Test".to_vec()),
                name: JvmBytes(name.to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
        }
    }

    /// The counterexample this slice exists for: **two throw sites of one block** enter the same
    /// handler, and each arrives with the locals of its own instruction.
    ///
    /// `local1` is an `int` at the first site and `null` at the second, `local2` is `5` at both, and
    /// `local3` is written only **after** the last site. The handler is entered with `Top`, `Int`
    /// and `Top`: a state built from the block's exit would carry the `null` of `local1` and the
    /// `5` of `local3`, and one built from the block's entry would carry `Top` for `local2`. The
    /// two sites are one canonical edge, which is why the logical inputs are not the edge count.
    #[test]
    fn two_throw_sites_of_one_block_enter_the_handler_with_their_own_locals() {
        let fixture = fixture_body(
            &[
                0x08, // 0: iconst_5
                0x3d, // 1: istore_2    local2 = 5
                0x03, // 2: iconst_0
                0x3c, // 3: istore_1    local1 = 0
                0x03, 0x03, 0x6c, // 4: iconst_0, 5: iconst_0, 6: idiv  (site 1)
                0x57, // 7: pop
                0x01, // 8: aconst_null
                0x4c, // 9: astore_1    local1 = null
                0x03, 0x03, 0x6c, // 10: iconst_0, 11: iconst_0, 12: idiv  (site 2)
                0x57, // 13: pop
                0x08, // 14: iconst_5
                0x3e, // 15: istore_3    local3 = 5, written after both sites
                0xb1, // 16: return
                0x4b, // 17: astore_0    (the handler entry)
                0xb1, // 18: return
            ],
            4,
        );
        let synthetic = with_handlers(
            fixture.facts,
            fixture.pool,
            vec![ExceptionHandlerFact {
                ordinal: 0,
                start_bci: 0,
                end_bci: 17,
                handler_bci: 17,
                catch_type_index: None,
            }],
        );
        assert_eq!(
            synthetic
                .canonical
                .edges
                .iter()
                .filter(|edge| matches!(edge.kind, CanonicalEdgeKind::Exception { .. }))
                .count(),
            1,
            "the graph aggregates the block's two throw sites into one exception edge"
        );
        let table = frames_or_panic(synthetic_frames(&synthetic, b"()V"));
        let handler = entry_of(&table, 17);
        assert_eq!(
            handler.locals[1],
            Value::Top,
            "`int` at the first site, `null` at the second: the merge of the two sites and not the \
             block's exit state"
        );
        assert_eq!(
            handler.locals[2],
            Value::Int,
            "`5` at both sites, and `Top` in the block's entry state"
        );
        assert_eq!(
            handler.locals[3],
            Value::Top,
            "`5` is written only after the last throw site, which therefore does not see it"
        );
        assert_eq!(
            handler.stack,
            vec![Value::Ref(RefType::Unknown)],
            "a catch-all record: one conservative unknown reference and nothing else"
        );
        assert_eq!(
            handler.inputs,
            vec![
                LogicalInput {
                    from: CanonicalBlockId {
                        bci: 0,
                        path: Vec::new(),
                    },
                    exception: Some(0),
                    throw_site: Some(6),
                },
                LogicalInput {
                    from: CanonicalBlockId {
                        bci: 0,
                        path: Vec::new(),
                    },
                    exception: Some(0),
                    throw_site: Some(12),
                },
            ],
            "one logical input per throw site: a value flow over this handler has two inputs here \
             and not the edge's one"
        );
    }

    /// The locals a throw site contributes are the state **at** the instruction, before it takes
    /// effect.
    ///
    /// The site list is the pass's own input, and this test states one on an `istore`: no
    /// instruction that may throw writes a local in this slice, so no body can show that difference
    /// by itself — 4.2's alias conversion is the one that will, and this is the snapshot point it
    /// has to respect. The record makes the difference observable now: the `istore` at BCI 1 writes
    /// `5` into local 1, so a snapshot taken *after* the instruction would hand the handler `5`
    /// instead of the `Top` local 1 still holds at it.
    #[test]
    fn a_throw_site_contributes_the_state_before_its_instruction_takes_effect() {
        let fixture = fixture_body(
            &[
                0x08, // 0: iconst_5
                0x3c, // 1: istore_1  (local1 = 5; the site this test states)
                0x03, 0x03, 0x6c, // 2: iconst_0, 3: iconst_0, 4: idiv  (a site of the body)
                0xb1, // 5: return
                0x4e, // 6: astore_3  (the handler entry)
                0xb1, // 7: return
            ],
            4,
        );
        let mut synthetic = with_handlers(
            fixture.facts,
            fixture.pool,
            vec![ExceptionHandlerFact {
                ordinal: 0,
                start_bci: 0,
                end_bci: 6,
                handler_bci: 6,
                catch_type_index: None,
            }],
        );
        let block = synthetic.canonical.blocks[0].id.clone();
        assert_eq!(
            block,
            CanonicalBlockId {
                bci: 0,
                path: Vec::new(),
            },
            "the body is one block, so the site and the transfer are the same node"
        );
        assert_eq!(
            synthetic.canonical.throw_sites.len(),
            1,
            "the body's own site is the `idiv` alone"
        );
        synthetic.canonical.throw_sites.push(CanonicalThrowSite {
            bci: 1,
            opcode: 0x3c,
            block,
            handlers: vec![0],
            origin: jarde_reader::model::OriginSet::default(),
        });
        let table = frames_or_panic(synthetic_frames(&synthetic, b"()V"));
        let handler = entry_of(&table, 6);
        assert_eq!(
            handler.locals[1],
            Value::Top,
            "the site at the `istore` is entered before the store: local 1 holds no readable value \
             there, and only the state after it holds `5`"
        );
        assert_eq!(
            handler.inputs.len(),
            2,
            "the injected site and the body's own site are two logical inputs of this handler"
        );
        assert_eq!(
            handler.inputs[0],
            LogicalInput {
                from: CanonicalBlockId {
                    bci: 0,
                    path: Vec::new(),
                },
                exception: Some(0),
                throw_site: Some(1),
            },
            "the site the test states is the first input, by its own BCI"
        );
    }

    /// The exception input of a constructor call is the state **at** the call: a slot written just
    /// before it is seen, a slot written only after it is not, and the handler holds the record's
    /// catch type.
    ///
    /// The receiver is the legal one now that the initialization conversions are here: the
    /// uninitialized `this` of a constructor, which the call converts on its normal completion. The
    /// `null` receiver this test used while the conversion was missing no longer reaches the
    /// handler at all — an `<init>` call on an initialized value has no transition in this pass,
    /// and the case below states that stop.
    #[test]
    fn the_exception_input_of_a_constructor_call_is_the_state_at_the_call() {
        let synthetic = synthetic_body(
            vec![
                plain(0, 0x08),    // iconst_5
                local(1, 0x3c, 1), // istore_1   local1 = 5, written before the call
                local(2, 0x2a, 0), // aload_0: the uninitialized `this` of the constructor
                call(3, 0xb7, 2),  // invokespecial #2 <init>()V (may raise)
                plain(6, 0x08),    // iconst_5
                local(7, 0x3d, 2), // istore_2   local2 = 5, written only after the call
                plain(8, 0xb1),    // return
                local(9, 0x4e, 3), // astore_3: the handler entry
                plain(10, 0xb1),   // return
            ],
            vec![
                method_ref_named(2, b"<init>", b"()V"),
                class_ref(3, b"java/lang/Throwable"),
            ],
            4,
            11,
        );
        let synthetic = with_handlers(
            synthetic.facts,
            synthetic.pool,
            vec![ExceptionHandlerFact {
                ordinal: 0,
                start_bci: 0,
                end_bci: 9,
                handler_bci: 9,
                catch_type_index: Some(3),
            }],
        );
        let table = frames_or_panic(constructor_frames(&synthetic, b"()V"));
        let handler = entry_of(&table, 9);
        assert_eq!(
            handler.locals[0],
            Value::UninitializedThis,
            "the state at the call holds the token the call was about to convert"
        );
        assert_eq!(
            handler.locals[1],
            Value::Int,
            "the call was entered with local 1 = 5"
        );
        assert_eq!(
            handler.locals[2],
            Value::Top,
            "local 2 is written only after the call, so the call's own state does not hold it"
        );
        assert_eq!(
            handler.stack,
            vec![Value::Ref(RefType::Named {
                name: b"java/lang/Throwable".to_vec(),
                loader: Box::new(LoaderId("app".to_string())),
            })],
            "the record's catch type is the entry's single reference"
        );
        assert_eq!(
            handler.inputs,
            vec![LogicalInput {
                from: CanonicalBlockId {
                    bci: 0,
                    path: Vec::new(),
                },
                exception: Some(0),
                throw_site: Some(3),
            }],
            "the call is the one throwing instruction of this block"
        );
    }

    /// Two clones of one shared subroutine each run their own `new`, and the two tokens are not the
    /// same value.
    ///
    /// The body's subroutine is entered by the `jsr` at BCI 0 and by the `jsr` at BCI 3, so the
    /// canonical graph clones the block that holds the `new` once per call site. Both clones map
    /// back to the **same** original BCI — which is exactly why the token cannot be keyed by it —
    /// and what tells the two allocations apart is the canonical block the `new` runs in. Each
    /// call's continuation is where the token lands, and the two continuations differ only in it,
    /// so the assertion is on the identity the token carries; neither body calls an `<init>`, so
    /// no conversion is involved in what they leave behind.
    #[test]
    fn two_clones_of_one_subroutine_get_two_new_sites() {
        let synthetic = jsr_body(
            vec![
                branch(0, 0xa8, 7), // jsr 7: the first call site, continuation BCI 3
                branch(3, 0xa8, 4), // jsr 7: the second call site, continuation BCI 6
                plain(6, 0xb1),     // return
                local(7, 0x4b, 0),  // astore_0: the subroutine entry, local0 = the return address
                call(8, 0xbb, 2),   // new Test
                local(11, 0x4c, 1), // astore_1: local1 = the new-site token
                local(12, 0xa9, 0), // ret 0
            ],
            vec![class_ref(2, b"Test")],
            2,
            13,
        );
        let table = frames_or_panic(synthetic_frames(&synthetic, b"()V"));
        let site = |bci: u32| match &entry_of(&table, bci).locals[1] {
            Value::Uninitialized { new_site } => new_site.clone(),
            other => panic!("local 1 at BCI {bci} holds the `new` token, found {other:?}"),
        };
        let (first, second) = (site(3), site(6));
        assert_eq!(
            (first.bci, second.bci),
            (8, 8),
            "both clones map back to one original `new`"
        );
        assert_ne!(
            first, second,
            "the BCI is the same and the token is not: the canonical site tells them apart"
        );
        assert_eq!(
            (first.block.path, second.block.path),
            (vec![0], vec![3]),
            "each token carries the call path of the clone it was run in"
        );
        assert_eq!(
            (first.block.bci, second.block.bci),
            (7, 7),
            "and both name the original block they were cloned from"
        );
    }

    /// Every block records the logical inputs its entry state was merged from: the source block,
    /// and the throw site when the input arrives through an exception edge.
    #[test]
    fn every_block_records_the_logical_inputs_it_was_merged_from() {
        let (mut code, join) = diamond(&[], &[0x03, 0x3c], &[0x01, 0x4b]);
        code.push(0xb1); // return
        let fixture = fixture_body(&code, 4);
        let table = frames_or_panic(frames_of(&fixture));
        assert!(
            entry_of(&table, 0).inputs.is_empty(),
            "the method's entry block has no input"
        );
        assert_eq!(
            entry_of(&table, join).inputs,
            vec![
                LogicalInput {
                    from: CanonicalBlockId {
                        bci: 4,
                        path: Vec::new(),
                    },
                    exception: None,
                    throw_site: None,
                },
                LogicalInput {
                    from: CanonicalBlockId {
                        bci: 9,
                        path: Vec::new(),
                    },
                    exception: None,
                    throw_site: None,
                },
            ],
            "the join names both arms, with no throw site: a plain transfer is not an exception"
        );
    }

    /// The check the publish step runs on every block, driven with the two readings of one fact
    /// **directly**: the records a block holds against the contributions its merge saw.
    ///
    /// A folded record list is the defect the grouping key of [`run`] used to have — two exception
    /// edges of one source into one handler, and one group left for both of them — and it is a
    /// legal body of the graph, so no body can be written that makes this check fail while the pass
    /// is correct. It is therefore pinned here: the same list under one key is refused, the same
    /// list under the graph's two keys is accepted, and the count of the edges is what the refusal
    /// names. A check that a mutation turns into `debug_assert!`, or that someone relaxes into "the
    /// list is non-empty", fails this test long before the phase behind reports the bytes as
    /// self-contradictory.
    #[test]
    fn a_record_list_that_folded_two_edges_into_one_group_is_refused() {
        let source = CanonicalBlockId {
            bci: 4,
            path: Vec::new(),
        };
        let handler = CanonicalBlockId {
            bci: 11,
            path: Vec::new(),
        };
        let record = LogicalInput {
            from: source.clone(),
            exception: Some(0),
            throw_site: Some(5),
        };
        // The two edges the graph holds: one handler named by two records, which is what makes
        // them two edges of one source and one target.
        let contributions: BTreeMap<EdgeIdentity, u64> = BTreeMap::from([
            (
                (
                    source.clone(),
                    CanonicalEdgeKind::Exception { handler_ordinal: 0 },
                ),
                1,
            ),
            (
                (
                    source.clone(),
                    CanonicalEdgeKind::Exception { handler_ordinal: 2 },
                ),
                1,
            ),
        ]);
        let folded: BTreeMap<EdgeKey, Vec<LogicalInput>> =
            BTreeMap::from([((source.clone(), Some(2)), vec![record.clone()])]);
        let message = match records_are_the_contributions(&handler, &folded, &contributions) {
            Err(Problem::Inconsistent(message)) => message,
            Err(_) => panic!(
                "a fold is refused through the module's own contradiction code, not as a budget \
                 stop or a boundary of this build"
            ),
            Ok(()) => panic!("a folded record list must be refused"),
        };
        assert!(
            message.contains("1 logical input record(s)") && message.contains("2 contribution(s)"),
            "the refusal states both readings of the fact: {message}"
        );
        assert!(
            message.contains("2 edge(s)"),
            "and the graph's own count, which is what the grouping must answer for: {message}"
        );
        let kept: BTreeMap<EdgeKey, Vec<LogicalInput>> = BTreeMap::from([
            ((source.clone(), Some(0)), vec![record.clone()]),
            ((source.clone(), Some(2)), vec![record]),
        ]);
        assert!(
            records_are_the_contributions(&handler, &kept, &contributions).is_ok(),
            "two groups answer for the graph's two edges"
        );
    }
}
