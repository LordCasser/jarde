//! The vocabulary the presentation is written in, and the only two facts the payload does not
//! carry: the method's identity and its debug names.
//!
//! # Where the facts come from (P3 1.3b)
//!
//! A presentation cannot be written without knowing which local an instruction reads, what a
//! constant pushes, whether a branch transfers when its value is zero or when it is non-zero. Those
//! are **decode facts**: the class file stated them once, and the `raw_facts` pass of the analysis
//! run read them. They are not in this module and they are not the caller's to supply any more —
//! they travel inside [`jarde_jvm::method_ir::MethodIr`], with the tables of the very run that
//! decoded them, and [`crate::decode`] turns them into the [`Operation`]s below. One run, one
//! source: there is no parameter left through which a caller could hand in a second opinion about
//! an `ifeq`'s polarity, a `*load`'s slot or a constant's value.
//!
//! What this module still holds is what the payload genuinely does not carry:
//!
//! * the method's **identity** (name, descriptor, parameter slots) — the request named it, and the
//!   payload was built for one method without repeating its name;
//! * its **debug names** — the `LocalVariableTable`/`MethodParameters` evidence, which is an
//!   attribute the recovery driver reads beside the body. Absent debug metadata is a stated fact
//!   (`None` per slot, or no list at all) and [`crate::names`] then derives an ordinal name.
//!
//! # The boundary this keeps
//!
//! [`Operation`] is deliberately *not* a statement. It says "this BCI reads local 1", never "this
//! BCI is an assignment to `count`" — the spelling, the statement shape, the declaration, the
//! negation of a branch's polarity (the emitter writes the fall-through condition, which is the
//! negation of the jump sense) and the origin of every node are all decided above this seam. An
//! operation the subset does not model — a field access, an array operation, a conversion — is
//! [`Operation::Other`] and makes the statement it belongs to unrenderable, which is a declared
//! fallback with a diagnostic and never an invented expression.

use std::collections::BTreeMap;

use jarde_jvm::method_ir::parameter_positions;
use jarde_reader::classfile::{DescriptorKind, descriptor_facts};
use jarde_reader::model::PhysicalMethodId;
use serde::Serialize;

use crate::ast::Type;
use crate::lambda::type_of_component;
use crate::names::DebugLocal;

/// The access-flag bit a class or a member sets when it is `public`.
pub const ACC_PUBLIC: u16 = 0x0001;

/// The access-flag bit a member sets when it is `private`.
pub const ACC_PRIVATE: u16 = 0x0002;

/// The access-flag bit a class sets on a member that is `static`.
pub const ACC_STATIC: u16 = 0x0008;

/// The access-flag bit a class sets on the bridge method a compiler generated.
pub const ACC_BRIDGE: u16 = 0x0040;

/// The access-flag bit a class sets on a member a compiler generated.
pub const ACC_SYNTHETIC: u16 = 0x1000;

/// The access-flag bit a **class** sets on itself when it is an interface (JVMS 4.1).
pub const ACC_INTERFACE: u16 = 0x0200;

/// The access-flag bit a class sets on itself when it is an annotation type, which JVMS 4.1 defines
/// as an interface with one extra constraint.
pub const ACC_ANNOTATION: u16 = 0x2000;

/// The access-flag bit a class or a member sets when it is `abstract`.
pub const ACC_ABSTRACT: u16 = 0x0400;

/// What a class file says about the method whose body is being presented.
///
/// Identity: the name and descriptor are evidence the presentation quotes, and the parameter
/// count is what tells an ordinal name whether it belongs to a parameter or to a local.
///
/// The member's **access flags** are here too, and they stay optional because a caller may be the
/// one that read the declaration: the entry point fills them from the payload's own member
/// declaration (P3 3.1), while a caller that read no header, or a run that published no member
/// declaration, states nothing. The difference matters to exactly one rule — a **bridge** is a
/// member the compiler declared as one ([`ACC_BRIDGE`]), so a run that states no flags has no
/// bridge to present and says so by having no verdict to record, rather than by guessing from the
/// body's shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodFacts {
    name: String,
    descriptor: String,
    parameters: u16,
    access_flags: Option<u16>,
    declaring_class: Option<DeclaringClass>,
}

impl MethodFacts {
    /// One method's identity: its name, its descriptor and its number of parameter slots.
    pub fn new(name: impl Into<String>, descriptor: impl Into<String>, parameters: u16) -> Self {
        Self {
            name: name.into(),
            descriptor: descriptor.into(),
            parameters,
            access_flags: None,
            declaring_class: None,
        }
    }

    /// The same facts with the member's access flags, exactly as the class declared them.
    pub fn with_access_flags(mut self, access_flags: u16) -> Self {
        self.access_flags = Some(access_flags);
        self
    }

    /// The same facts with the class that declares the member, as the run's own read of that class
    /// stated it (P3 2.3; the declaring-class handoff).
    ///
    /// Like the member's flags this is a **declaration fact** of the class file the body was read
    /// from, and it stays optional for the same reason: the entry point fills it from the payload's
    /// own member declaration — the class's raw internal name and its access flags, both taken from
    /// the one header read that also decoded the body — while a caller that read no header, or a
    /// run that published no member declaration, states nothing. The rules that read one —
    /// `declaration@1`, `init@1` and `field@1` — state it as a declared precondition and refuse
    /// with the missing fact named when it is absent.
    pub fn with_declaring_class(mut self, declaring_class: DeclaringClass) -> Self {
        self.declaring_class = Some(declaring_class);
        self
    }

    /// The type each **parameter slot** holds, as this method's own descriptor states it.
    ///
    /// This is the fact the frames cannot state (P3-R5): a `boolean`, a `byte`, a `char` and a
    /// `short` are one slot shape, so a body read on its own writes `b != 0` for a `boolean`
    /// parameter — text the member's own signature refuses to compile. The descriptor names the
    /// primitive outright, so a run that reads it can write `!b`/`b` where the value is tested and
    /// declare a local filled from it as `boolean`.
    ///
    /// The map is keyed by the slot the parameter occupies, which is the same numbering
    /// [`Self::parameters`] counts: an instance method's receiver holds slot 0, and whether this
    /// method has one is a declaration fact the caller states. A run that states **neither** the
    /// flags nor a count that places the parameters (a descriptor whose slots could start at 0 or at
    /// 1 and a count that agrees with both) states no type at all rather than guessing one.
    ///
    /// The descriptor is read once, through the reader's own facts, and the slots are the JVM
    /// layer's own derivation of them ([`parameter_positions`]): an array of a `long` or a `double`
    /// is one slot, so the parameter after it is placed where the bytes really put it. A reference —
    /// an object type or an array of either — keeps the descriptor's exact Java type, which is also
    /// the fact a call argument uses to preserve overload selection.
    pub fn parameter_types(&self) -> BTreeMap<u16, Type> {
        let Ok(facts) = descriptor_facts(self.descriptor.as_bytes(), DescriptorKind::Method) else {
            return BTreeMap::new();
        };
        let is_static = match self.access_flags {
            Some(flags) => flags & ACC_STATIC != 0,
            None => {
                // The caller stated no flags: the count it did state places the parameters when it
                // agrees with exactly one of the two layouts.
                let Some(described) = facts.parameter_slots() else {
                    return BTreeMap::new();
                };
                if self.parameters == described {
                    true
                } else if self.parameters == described.saturating_add(1) {
                    false
                } else {
                    return BTreeMap::new();
                }
            }
        };
        let Some(positions) = parameter_positions(&facts, is_static) else {
            return BTreeMap::new();
        };
        let mut types = BTreeMap::new();
        for (component, slot) in facts.parameters().iter().zip(positions) {
            let Some(ty) = type_of_component(component) else {
                return BTreeMap::new();
            };
            types.insert(slot, ty);
        }
        types
    }

    /// The class the member is declared in, when the caller stated it.
    pub fn declaring_class(&self) -> Option<&DeclaringClass> {
        self.declaring_class.as_ref()
    }

    /// The access flags the class declares for this member, when the caller stated them.
    pub fn access_flags(&self) -> Option<u16> {
        self.access_flags
    }

    /// Whether the class declares this member as a bridge a compiler generated.
    ///
    /// This is a *declaration* fact, never a guess from the body: a member whose body looks like a
    /// forward but which the class did not declare as a bridge is an ordinary method, and the
    /// `bridge@1` rule does not touch it.
    pub fn is_bridge(&self) -> bool {
        matches!(self.access_flags, Some(flags) if flags & ACC_BRIDGE != 0)
    }

    /// Whether this member's slot 0 holds a receiver, which JVMS 4.10.1.9 puts there.
    ///
    /// The member's own `ACC_STATIC` bit is the whole answer: an instance method and a constructor
    /// take a receiver, a `static` method and a static initializer do not. Nothing else is read —
    /// not the descriptor, not the number of parameter slots, and not a debug name that happens to
    /// spell `this` — because the presentation writes `this` for exactly this slot and a keyword
    /// written into a member that never had a receiver would be a body no compiler accepts.
    ///
    /// A caller that stated no flags states no receiver either: the run writes `arg<slot>`/the
    /// debug name for slot 0 then, which is the same "no fact, no claim" answer
    /// [`Self::parameter_types`] gives the flagless case. Unlike the slot *placement* that method
    /// derives from a stated count, a receiver is an identity only the flags state, so an absent
    /// answer here stays absent.
    pub fn has_receiver(&self) -> bool {
        matches!(self.access_flags, Some(flags) if flags & ACC_STATIC == 0)
    }

    /// The method's own name as the class file spells it.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The method's descriptor as the class file spells it.
    pub fn descriptor(&self) -> &str {
        &self.descriptor
    }

    /// The number of parameter slots, `this` included when the caller counted it.
    pub fn parameters(&self) -> u16 {
        self.parameters
    }
}

/// A constant the class file pushes, as the decoding layer read it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConstantValue {
    /// An `int`-shaped constant; a `boolean` is one of these, and the presentation writes it as an
    /// `int` because the bytecode does not say which of the two the source had.
    Int(i64),
    /// A `long` constant, which the presentation writes with its `L` suffix.
    Long(i64),
    /// A `String` constant, as the constant-pool entry spells it; escaping happens in the emitter.
    String(String),
    /// `aconst_null`.
    Null,
    /// A type stated by one `CONSTANT_Class` entry reached by `ldc` or `ldc_w`, in the Java
    /// spelling this layer proved it can write, and the index of that entry in the same class's
    /// constant pool.
    Class { ty: String, pool_index: u16 },
}

/// The arithmetic a bytecode instruction performs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

/// The direction of an integral JVM shift; its opcode also states the left/result width.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShiftOp {
    Left,
    Right,
    UnsignedRight,
}

/// The integral or non-short-circuit boolean operation a bitwise bytecode instruction performs.
///
/// The `i*`/`l*` opcode pair states the computational width on the frame value; this fact keeps the
/// shared operator without claiming whether an int-shaped value is a Java `int` or `boolean`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitwiseOp {
    And,
    Or,
    Xor,
}

/// The JVM numeric comparison instruction that produces a signed `-1`/`0`/`1` result.
///
/// The floating-point variants retain the instruction's unordered (`NaN`) bias. This is a decode
/// fact, not a Java condition: the condition builder combines it with the one proven zero branch
/// that consumes the result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericComparisonOp {
    /// `lcmp`, which has no unordered value.
    Long,
    /// `fcmpl`, which produces `-1` for unordered operands.
    FloatLess,
    /// `fcmpg`, which produces `1` for unordered operands.
    FloatGreater,
    /// `dcmpl`, which produces `-1` for unordered operands.
    DoubleLess,
    /// `dcmpg`, which produces `1` for unordered operands.
    DoubleGreater,
}

/// The sense of a conditional branch: whether it transfers when the condition holds or fails.
///
/// Both polarities are needed and neither is derivable from structure: an `if (a) … else …` and an
/// `if (!a) … else …` have the same graph and the same arms. The recovering layer writes the
/// condition under which control falls through, which is the *negation* of this sense — see
/// [`crate::region`] for the rule and `crate::build` for where it is applied — and a loop writes
/// whichever of the two senses is the one that keeps iterating.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompareOp {
    /// One value: transfer when it is zero (`ifeq`).
    JumpIfZero,
    /// One value: transfer when it is non-zero (`ifne`).
    JumpIfNotZero,
    /// One reference: transfer when it is null (`ifnull`).
    JumpIfNull,
    /// One reference: transfer when it is not null (`ifnonnull`).
    JumpIfNotNull,
    /// Two values: transfer when they are equal (`if_icmpeq`, `if_acmpeq`).
    JumpIfSame,
    /// Two values: transfer when they differ (`if_icmpne`, `if_acmpne`).
    JumpIfDifferent,
    /// One value: transfer when it is below zero (`iflt`).
    JumpIfNegative,
    /// One value: transfer when it is at or above zero (`ifge`).
    JumpIfNotNegative,
    /// One value: transfer when it is above zero (`ifgt`).
    JumpIfPositive,
    /// One value: transfer when it is at or below zero (`ifle`).
    JumpIfNotPositive,
    /// Two values: transfer when the first is below the second (`if_icmplt`).
    JumpIfLess,
    /// Two values: transfer when the first is at or below the second (`if_icmple`).
    JumpIfLessOrEqual,
    /// Two values: transfer when the first is above the second (`if_icmpgt`).
    JumpIfGreater,
    /// Two values: transfer when the first is at or above the second (`if_icmpge`).
    JumpIfGreaterOrEqual,
}

impl CompareOp {
    /// Whether the instruction reads two values rather than one.
    pub fn reads_two(self) -> bool {
        matches!(
            self,
            Self::JumpIfSame
                | Self::JumpIfDifferent
                | Self::JumpIfLess
                | Self::JumpIfLessOrEqual
                | Self::JumpIfGreater
                | Self::JumpIfGreaterOrEqual
        )
        // `JumpIfNegative`/`JumpIfNotNegative`/`JumpIfPositive`/`JumpIfNotPositive` read one value
        // and compare it against zero: they are the `iflt`/`ifge`/`ifgt`/`ifle` family, not the
        // `if_icmp*` one, and reading two values for them would ask the value flow for one the
        // instruction never read.
    }
}

/// How an invocation reaches its target, which decides the shape of the call the presentation
/// writes (`Type.name(...)` for a static, `receiver.name(...)` otherwise).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvokeKind {
    Static,
    Virtual,
    Special,
    Interface,
}

/// One symbolic reference an invocation names.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CallTarget {
    kind: InvokeKind,
    owner: String,
    name: String,
    descriptor: String,
    interface_reference: bool,
}

impl CallTarget {
    /// One invocation's target: how it is reached, its owner as the class file spells it (internal
    /// form, `java/lang/Object`), its name and descriptor, and the pool entry's interface flag.
    pub fn new(
        kind: InvokeKind,
        owner: impl Into<String>,
        name: impl Into<String>,
        descriptor: impl Into<String>,
        interface_reference: bool,
    ) -> Self {
        Self {
            kind,
            owner: owner.into(),
            name: name.into(),
            descriptor: descriptor.into(),
            interface_reference,
        }
    }

    /// How the invocation reaches the target.
    pub fn kind(&self) -> InvokeKind {
        self.kind
    }

    /// The owner in internal form, exactly as the class file states it.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// The member name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The member descriptor.
    pub fn descriptor(&self) -> &str {
        &self.descriptor
    }

    /// Whether the pool entry was a `CONSTANT_InterfaceMethodref`.
    pub fn is_interface_reference(&self) -> bool {
        self.interface_reference
    }
}

/// One `invokedynamic` call site, as the class's own pool states it.
///
/// This is the site's **identity**, and nothing about its shape: the entry names the
/// `BootstrapMethods` entry the site resolves against and the name and descriptor the site
/// presents. For a lambda site that name is the SAM's method name and that descriptor is the
/// captured variables' types followed by the functional interface — but *that a site is a lambda at
/// all* is not decided here, and not decided by this type: it is decided against the class's
/// bootstrap table and the implementation handle ([`crate::lambda`]), and a site whose bootstrap is
/// anything else keeps this identity and no lambda presentation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DynamicSite {
    cp: u16,
    bootstrap_index: u16,
    name: String,
    descriptor: String,
}

impl DynamicSite {
    /// One dynamic site from the pool entry that states it.
    pub fn new(
        cp: u16,
        bootstrap_index: u16,
        name: impl Into<String>,
        descriptor: impl Into<String>,
    ) -> Self {
        Self {
            cp,
            bootstrap_index,
            name: name.into(),
            descriptor: descriptor.into(),
        }
    }

    /// The constant-pool index of the site's own `InvokeDynamic` entry — the use site's own
    /// reference, which a report reads back rather than re-deriving from the BCI.
    pub fn cp(&self) -> u16 {
        self.cp
    }

    /// The `BootstrapMethods` entry this site names, or `None`-like when the class states a table
    /// that does not reach it (the caller checks the table; this is the index alone).
    pub fn bootstrap_index(&self) -> u16 {
        self.bootstrap_index
    }

    /// The name the site presents; for a lambda site, the SAM method's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The descriptor the site presents; for a lambda site, the captured variables' types and the
    /// functional interface the site instantiates.
    pub fn descriptor(&self) -> &str {
        &self.descriptor
    }
}

/// What one bytecode instruction does, as far as the presentation needs to know.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    /// Pushes a constant; the next store that reads it writes a literal.
    Push(ConstantValue),
    /// Reads a local slot.
    Load { slot: u16 },
    /// Writes a local slot.
    Store { slot: u16 },
    /// Combines the two values it reads (below the value it reads first).
    Arithmetic { op: ArithmeticOp },
    /// Shifts an int-shaped or long left value by an int-shaped distance.
    Shift { op: ShiftOp },
    /// Combines two int-shaped or long values using the instruction's bitwise operator.
    ///
    /// The instruction does not distinguish Java `int` from `boolean`; that evidence belongs to
    /// the value consumers and the declaration plan.
    Bitwise { op: BitwiseOp },
    /// Negates the one numeric value it reads (`ineg`, `lneg`, `fneg` or `dneg`).
    Negate,
    /// Converts the one numeric value it reads, retaining the JVM operand shape and Java result type.
    ///
    /// `source` is the primitive category named by the opcode (`int`, `long`, `float` or
    /// `double`), not a claim that an int-shaped value is necessarily an `int`: the value reader
    /// checks its presented type and accepts byte/short/char where the JVM uses the int category,
    /// while refusing `boolean`. `target` is the explicit Java type the opcode produces, including
    /// byte/char/short for `i2b`/`i2c`/`i2s` even though their frame result remains int-shaped.
    PrimitiveConversion { source: Type, target: Type },
    /// Adds a signed amount to a local slot, in place (`iinc`): one statement's worth of effect
    /// that reads and writes the same slot, which is why it is neither a load nor a store.
    Increment { slot: u16, amount: i32 },
    /// Transfers on a condition built from the values it reads, to the BCI the decode states.
    ///
    /// The target is a decode fact, not a shape: a conditional branch jumps to the address its own
    /// operand names, and that address is what tells the fall-through arm (the successor that is
    /// not the target) from the transferred one. Reading it instead of assuming "the further
    /// successor is the target" is what lets a backward branch — the condition of a loop — be
    /// presented at all.
    Comparison { op: CompareOp, target: u32 },
    /// Produces a signed numeric comparison result, which may be composed with a zero branch.
    NumericComparison { op: NumericComparisonOp },
    /// A multi-way transfer: the keys the decode enumerated, each with the BCI it transfers to,
    /// plus the BCI of the no-match case.
    ///
    /// Keys are kept as signed values because that is what the payload states: a `tableswitch`
    /// enumerates a range, a `lookupswitch` a sorted list of `(match, offset)` pairs, and nothing
    /// below this layer decides how a key is spelled in Java.
    Switch {
        cases: Vec<(i64, u32)>,
        default: u32,
    },
    /// A plain transfer the *structure* already stands for — a `goto` inside a recovered region.
    ///
    /// It is neither a value nor an effect on the program's state, so it becomes no statement; and it
    /// is not [`Self::Other`] either, because the structure layer did model what it does (it is an
    /// edge of the region it belongs to), which is exactly the difference between "this layer does
    /// not model the operation" and "the operation leaves no statement behind".
    Transfer,
    /// Invokes the target it names, with the receiver and arguments it reads.
    Invoke(CallTarget),
    /// Allocates an uninitialized instance of the class the instruction names (`new`).
    ///
    /// The name is the internal form the class's own pool states (`java/lang/StringBuilder`), and
    /// it is the only thing this instruction says: an allocation is not yet an instance, and until
    /// a constructor has run on it the presentation must not write anything for it.
    Allocate { ty: String },
    /// Duplicates the value on top of the stack (`dup`).
    ///
    /// It is not a value of its own and not an effect on the program's state: it is the shape a
    /// chain that builds one instance and then uses it twice is written in, which is why the
    /// concatenation rule reads it (the instance the `new` produced is the one every `append` and
    /// the `toString` of the chain are called on).
    Duplicate,
    /// Reads or writes one field, with the symbol the class's own pool states.
    ///
    /// This is a modelled fact and not a presentation: which field an accessor reads, and whether
    /// the access is a read or a write on an instance or on a type, is what decides whether a
    /// synthetic member's body is the pure forwarding the `accessor@1` rule presents. Nothing here
    /// writes a field access of its own — [`crate::build`] does that, and only for a call site
    /// whose callee's body this fact set was used to verify.
    Field {
        access: FieldAccess,
        /// Whether the instruction names a field of the class itself rather than of an instance.
        is_static: bool,
        /// The owner in internal form, exactly as the class file states it.
        owner: String,
        name: String,
        descriptor: String,
    },
    /// Checks that the value it reads is assignable to the class it names (`checkcast`).
    ///
    /// A check is not an erasure: dropping one changes what the method does unless the value's own
    /// type already proves the check can neither fail nor change the value. That proof is the
    /// `bridge@1` rule's, and a cast no rule has claimed is [`Self::Other`]'s business — quoted,
    /// never silently dropped.
    CheckCast { ty: String },
    /// Tests one reference against the named CP class, producing a JVM int-shaped boolean.
    InstanceOf { ty: String },
    /// A dynamic call site: it reads the captured values the site's descriptor names and produces
    /// the instance the descriptor returns.
    ///
    /// The **shape** is not here. Whether this site is a lambda, a method reference or something
    /// this layer must not touch is decided from the class's bootstrap table, the factory method
    /// handle and the implementation handle ([`crate::lambda`], rule `lambda@1`), because a site
    /// with an arbitrary bootstrap must never be presented as one of them (A04).
    InvokeDynamic(DynamicSite),
    /// Reads one element of an `int`-shaped array (`iaload`): the read the `switch` a compiler
    /// builds for an enum reads its dispatch table with.
    ///
    /// The *shape* that makes one read a switch's dispatch is [`crate::enumswitch`]'s reading, and
    /// that rule's claim is unchanged — which is why this variant is the read it names and not the
    /// whole array-reading family: a dispatch table is an `int[]`, so the `baload`/`caload`/
    /// `saload` that share this value shape are no candidate of that rule and are
    /// [`Self::ArrayElementLoad`]s. A read no rule claimed is the ordinary subscript
    /// [`crate::build`] writes.
    ArrayLoad,
    /// Reads one element of an array whose element type the **opcode** states — or, with `None`,
    /// whose element type only the array's own type can state.
    ///
    /// The variant carries the element type for the reads whose opcode names it (`laload`/`faload`/
    /// `daload` name theirs outright), for the three that share `iaload`'s int-sized shape — an
    /// `int`, which is the whole of what their opcode says: JVMS 2.11.1 gives a `boolean`, a `byte`,
    /// a `char`, a `short` and an `int` one value shape and one slot, so `[Z` and `[B` are both read
    /// with `baload` — and `None` for `aaload`, whose element is a fact of the array's own type
    /// (`[Ljava/lang/String;` reads a `java.lang.String`, `[[I` an `int[]`) and never of the
    /// instruction. What the **array's** own type states is [`crate::build`]'s refinement over
    /// this; `None` is *no type claimed* and not a type to guess.
    ArrayElementLoad { element: Option<Type> },
    /// Writes one element of an array (`iastore`, `lastore`, …, `aastore`).
    ///
    /// The element type is what the written element has to meet, and it comes from the same two
    /// places a read's does: the opcode states it for seven of the eight stores, and `None` is
    /// `aastore`, whose element is a reference the array's own type names. `Some(Int)` for
    /// `bastore`/`castore`/`sastore` is the same four-int rule [`Self::ArrayLoad`] states, and
    /// [`crate::build`] refines it from the array's own type where the frames state one.
    ArrayStore { element: Option<Type> },
    /// Reads the length of the array it reads (`arraylength`).
    ///
    /// A modelled fact and not a presentation: the count a loop's test reads is what makes the test
    /// worth writing, and whether a particular `arraylength` sits in a position this layer presents
    /// is [`crate::build`]'s question. `.length` is the only text it has.
    ArrayLength,
    /// Allocates one array (`newarray`, `anewarray` and `multianewarray`).
    ///
    /// `element` is the **element** type the creation states — `newarray`'s `atype` code,
    /// `anewarray`'s pool class, and the component the `multianewarray` descriptor names —
    /// `dimensions` is how many lengths the instruction reads off the stack, and
    /// `total_dimensions` is the complete array rank from the instruction/constant-pool facts.
    /// JVMS 6.5 makes those facts explicit: `anewarray` allocates one dimension even when its
    /// component is itself an array, while `multianewarray`'s operand allocates only a prefix of
    /// the pool class's rank.
    NewArray {
        element: Type,
        dimensions: u8,
        total_dimensions: u8,
    },
    /// Enters or leaves the monitor of the object it reads (`monitorenter`/`monitorexit`).
    ///
    /// Modelled because the `monitor@1` rule reads it (P3 2.4): a `synchronized` statement *is* one
    /// enter and the exits that pair with it, and which of the two an instruction is decides the
    /// pairing the rule has to prove before it may write the statement. It is not a presentation of
    /// its own — a monitor instruction no rule claimed is quoted ([`Self::Other`]'s fate), never
    /// written as a lock this layer would have to invent.
    Monitor {
        /// Whether the instruction enters the monitor (`monitorenter`) or leaves it.
        enter: bool,
    },
    /// Throws the object it reads (`athrow`).
    ///
    /// Modelled for the same reason, and it is the fact the whole exceptional-path half of 2.4
    /// turns on: a guarded region's handler rethrows the exception it stored, and the `finally`
    /// shape javac emits ends each of its copies with one. Reading *which* object an `athrow`
    /// rethrows is what proves a handler preserves the primary rather than dropping it. A `throw`
    /// statement is not part of this subset: an `athrow` no rule claimed is quoted.
    Throw,
    /// Leaves the method.
    Return,
    /// Anything else: legal to read, not part of the provable subset.
    ///
    /// The variant exists so that "this layer has not modelled the operation yet" is a *stated*
    /// input rather than an unstated assumption. A statement built on one becomes a fallback with a
    /// diagnostic, which is what keeps a field access, an array store or a conversion from being
    /// printed as a guess before 2.x/3.x land.
    Other,
}

impl Operation {
    /// The sense and the target of one conditional branch, when this operation is one.
    ///
    /// Both are decode facts of the same instruction, and both are what a structure needs before it
    /// can write a condition: the sense says which way round the test is, the target says which of
    /// the two successors the branch transfers to.
    pub fn comparison(&self) -> Option<(CompareOp, u32)> {
        match self {
            Self::Comparison { op, target } => Some((*op, *target)),
            _ => None,
        }
    }

    /// The keys and the no-match target of one `switch`, when this operation is one.
    pub fn switch(&self) -> Option<(&[(i64, u32)], u32)> {
        match self {
            Self::Switch { cases, default } => Some((cases.as_slice(), *default)),
            _ => None,
        }
    }
}

/// Which way one field instruction goes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldAccess {
    /// `getfield`/`getstatic`: the instruction reads the field.
    Read,
    /// `putfield`/`putstatic`: the instruction writes the field.
    Write,
}

/// The class that declares the presented member: its own name and access flags, as the header read
/// of that class states them.
///
/// One recovery run presents **one body**, and its payload holds that body's graph, frames, names,
/// decode and pool. Of what the class file says about the **class** it carries exactly two facts —
/// the class's raw internal name (`this_class`) and its access flags — inside the member declaration
/// of the same header read that decoded the body (the declaring-class handoff). Everything else a
/// class states about itself is still not in the payload: not its superclass, not its `InnerClasses`
/// attribute, nor any other class-level attribute. The caller may also be the one that read the
/// member out of a header and states the two facts itself; the entry point fills them from the run's
/// own declaration instead. Either way they are handed over as **declaration** facts, never as a
/// verdict about what the class means.
///
/// What the rules that read it do with them:
///
/// * `declaration@1` states the member's declaration in the artifact's envelope, and that is where
///   "this is a `default` method of an interface" or "this is a static method of an interface" is
///   decided — from `ACC_INTERFACE` on the class plus the member's own flags;
/// * `init@1` compares the class's name with the owner of the constructor call a body makes on its
///   own `this`, which is the only thing that tells `super(…)` from `this(…)` (JVMS 4.9.2 allows an
///   instance initializer to call exactly those two);
/// * `field@1` compares it the same way for a write the body makes on that uninitialized `this`
///   before the call, which JVMS 4.10.1.9 lets reach only a `Fieldref` that names the class being
///   constructed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclaringClass {
    name: String,
    access_flags: u16,
}

impl DeclaringClass {
    /// One class's declaration: its name in the class file's own internal form (`p/Outer`) and its
    /// access flags, exactly as the header states them.
    pub fn new(name: impl Into<String>, access_flags: u16) -> Self {
        Self {
            name: name.into(),
            access_flags,
        }
    }

    /// The class's name in internal form, as `this_class` spells it.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The class's own access flags.
    pub fn access_flags(&self) -> u16 {
        self.access_flags
    }

    /// Whether the class declares itself an interface (or an annotation type, which JVMS 4.1 makes
    /// one kind of interface).
    pub fn is_interface(&self) -> bool {
        self.access_flags & (ACC_INTERFACE | ACC_ANNOTATION) != 0
    }
}

/// One reference type name as a frame or a pool states it, in the class file's internal form
/// (`java/lang/String`).
///
/// A frame keeps a reference type in whichever spelling its source had — a descriptor
/// (`Ljava/lang/String;`) when it came from one, an internal name when it came from a `Class` entry
/// — so two names are the same type only after this normalisation. Comparing the raw spellings
/// instead would refuse shapes that are in fact proven, which is the failure mode this helper
/// exists to keep out of [`crate::field`] and [`crate::init`].
pub fn internal_form(name: &str) -> &str {
    name.strip_prefix('L')
        .and_then(|rest| rest.strip_suffix(';'))
        .unwrap_or(name)
}

/// One member of the class the presented body belongs to, with its declaration and its body.
///
/// This is the evidence the `accessor@1` rule needs and the payload cannot hold: a synthetic
/// accessor is a **different member** of the same class, so the run that presents the caller's body
/// can only see it if the caller hands it over. What the caller hands over is not a verdict — it
/// is the same two things the payload holds for the presented body (the member's declaration facts
/// and its decoded body), and every judgement about that body is made in [`crate::accessor`].
///
/// # What one member states, and what it does not (P3 3.2)
///
/// The member's [`MemberBody::identity`] is the class file it was read from — the definition, by
/// digest and length — *and* the member's own name and descriptor as that read spells them. It is
/// not decoration: a name and a descriptor alone are a claim about *some* class, and the anchors a
/// presentation derives from this member's body have to say which class file their BCI and CP index
/// are coordinates in. The identity is the only thing a caller can hand over that answers both, and
/// it is what the facade builds one from a read of **one** definition — a member of any other class
/// file is not a member this type can hold.
///
/// [`MemberBody::name`] and [`MemberBody::descriptor`] spell that identity's bytes for the rule's
/// comparisons and messages; they are the identity's own bytes, in the spelling those need, and
/// never a second source.
///
/// A member the class declares **without a body** (abstract or native) is a member of this table
/// too, with [`MemberBody::code`] `None`: the class's declaration of it is evidence, and its body is
/// simply not there to read — which is a different statement from "this class declares no such
/// member", and the rule states the difference.
#[derive(Clone, Debug)]
pub struct MemberBody {
    owner: String,
    identity: PhysicalMethodId,
    access_flags: u16,
    code: Option<jarde_reader::classfile::MethodCodeFacts>,
}

impl MemberBody {
    /// One member of a class, exactly as the same read of that class stated it.
    pub fn new(
        owner: impl Into<String>,
        identity: PhysicalMethodId,
        access_flags: u16,
        code: jarde_reader::classfile::MethodCodeFacts,
    ) -> Self {
        Self {
            owner: owner.into(),
            identity,
            access_flags,
            code: Some(code),
        }
    }

    /// The same member, as a class that declares it without a body: the declaration and the flags
    /// are read, and no `Code` attribute is there.
    pub fn without_body(
        owner: impl Into<String>,
        identity: PhysicalMethodId,
        access_flags: u16,
    ) -> Self {
        Self {
            owner: owner.into(),
            identity,
            access_flags,
            code: None,
        }
    }

    /// The class the member is declared in, in internal form.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// The member's physical identity: the class-file definition it was read from, and its own name
    /// and descriptor as that read spells them.
    pub fn identity(&self) -> &PhysicalMethodId {
        &self.identity
    }

    /// The member's name, in the spelling the class file's bytes decode to.
    pub fn name(&self) -> String {
        String::from_utf8_lossy(&self.identity.name.0).into_owned()
    }

    /// The member's descriptor, in the spelling the class file's bytes decode to.
    pub fn descriptor(&self) -> String {
        String::from_utf8_lossy(&self.identity.descriptor.0).into_owned()
    }

    /// The member's access flags, as the class declared them.
    pub fn access_flags(&self) -> u16 {
        self.access_flags
    }

    /// The member's decoded body, or `None` when the class declares it without one.
    pub fn code(&self) -> Option<&jarde_reader::classfile::MethodCodeFacts> {
        self.code.as_ref()
    }
}

/// The members of one class, as the caller read them beside the presented body.
///
/// The caller that reads a body's facts has the class's declaration in hand (that is where its own
/// member name came from); handing the *other* members over is what makes a call to a synthetic
/// accessor decidable at all. The seam keeps the discipline of the payload: a member arrives as
/// its declaration and its **decoded body**, never as a claim about what it does, and
/// [`crate::accessor`] is the only place that reads a meaning into it.
///
/// Every member of one table is a member of **one class-file definition** ([`MemberBody::identity`]),
/// which since P3 3.2 is what binds the two things a rule reads from it: the definition's bytes are
/// the bytes the member was read from, and its constant pool is the pool its body's references were
/// decoded against. A member of another class file is not one this table can hold, and `owner` — the
/// name the *definition* declares for itself — is the name a rule compares a call site's owner
/// against.
#[derive(Clone, Debug)]
pub struct ClassMembers {
    owner: String,
    members: Vec<MemberBody>,
}

impl ClassMembers {
    /// The declared members of one class.
    pub fn new(owner: impl Into<String>, members: Vec<MemberBody>) -> Self {
        Self {
            owner: owner.into(),
            members,
        }
    }

    /// The class these members are declared in, in internal form.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Every member the caller handed over, in declaration order.
    pub fn members(&self) -> &[MemberBody] {
        &self.members
    }
}

/// Every fact the recovery run needs that the IR payload does not carry: the method's identity and
/// the debug names of its locals.
///
/// The decoded operations are **not** here. They are [`crate::decode`]'s reading of the payload:
/// the very instructions and operands one analysis run decoded, moved into
/// [`jarde_jvm::method_ir::MethodIr`] with the tables of that run. A caller can state a method's
/// name and its debug names because the payload does not hold them; it cannot state what an
/// instruction does, because the run that decoded the instruction already said so.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryFacts {
    method: MethodFacts,
    debug_locals: Vec<DebugLocal>,
}

impl RecoveryFacts {
    /// The facts of one method, with no debug names.
    pub fn new(method: MethodFacts) -> Self {
        Self {
            method,
            debug_locals: Vec::new(),
        }
    }

    /// The same facts with the debug records the body's `Code` attribute states, in declaration
    /// order.
    ///
    /// The records are **evidence**, one name over one range of bytecode each: a body compiled
    /// without debug metadata passes an empty list, which means "no evidence" and leaves every name
    /// to [`crate::names`]'s ordinal rule. What those records mean for the produced text — and in
    /// particular whether two records for one slot make it two variables (P3 3.4) — is decided by
    /// [`crate::reuse`] from the body's own uses, not by the caller that read them.
    pub fn with_debug_locals(mut self, debug_locals: Vec<DebugLocal>) -> Self {
        self.debug_locals = debug_locals;
        self
    }

    /// The method these facts describe.
    pub fn method(&self) -> &MethodFacts {
        &self.method
    }

    /// The debug records the body states, in declaration order; empty when it states none.
    pub fn debug_locals(&self) -> &[DebugLocal] {
        &self.debug_locals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_caller_states_identity_and_debug_names_and_nothing_about_the_body() {
        // The seam this test states: a caller can build these facts, and there is no way to state
        // an operation in them — no method to call, no field to set. Polarity, slot numbers and
        // constant values come from the payload's own decode ([`crate::decode`]) or from nowhere.
        let facts =
            RecoveryFacts::new(MethodFacts::new("add", "(II)I", 3)).with_debug_locals(vec![
                DebugLocal::over(0, "this", 0, 4),
                DebugLocal::over(1, "left", 0, 4),
            ]);
        assert_eq!(facts.method().name(), "add");
        assert_eq!(facts.method().descriptor(), "(II)I");
        assert_eq!(facts.method().parameters(), 3);
        assert_eq!(facts.debug_locals().len(), 2);
        assert_eq!(facts.debug_locals()[1].name(), "left");
        assert_eq!(facts.debug_locals()[1].range(), Some((0, 4)));
        assert_eq!(
            RecoveryFacts::new(MethodFacts::new("run", "()V", 1))
                .debug_locals()
                .len(),
            0,
            "a body without debug metadata states no name at all"
        );
    }

    #[test]
    fn the_arithmetic_and_comparison_vocabulary_is_stated_once() {
        assert!(CompareOp::JumpIfSame.reads_two());
        assert!(CompareOp::JumpIfGreaterOrEqual.reads_two());
        assert!(!CompareOp::JumpIfZero.reads_two());
        assert!(!CompareOp::JumpIfNotNull.reads_two());
    }

    #[test]
    fn the_receiver_is_decided_by_the_members_own_static_flag_and_by_nothing_else() {
        // JVMS 4.10.1.9: slot 0 holds the receiver exactly when the member is not `static`. A
        // constructor and an instance method take one; a `static` method and a static initializer do
        // not; and a caller that stated no flags states no receiver — the ordinal naming then covers
        // slot 0, which is the answer this layer gives instead of writing a `this` it cannot prove.
        assert!(
            MethodFacts::new("value", "()I", 1)
                .with_access_flags(ACC_PUBLIC)
                .has_receiver()
        );
        assert!(
            MethodFacts::new("<init>", "()V", 1)
                .with_access_flags(0)
                .has_receiver()
        );
        assert!(
            !MethodFacts::new("of", "(I)LH;", 1)
                .with_access_flags(ACC_PUBLIC | ACC_STATIC)
                .has_receiver()
        );
        assert!(
            !MethodFacts::new("<clinit>", "()V", 0)
                .with_access_flags(ACC_STATIC)
                .has_receiver()
        );
        assert!(!MethodFacts::new("value", "()I", 1).has_receiver());
        // The flag is read, never a name or a count: the same identity without the flags answers
        // `false`, and a `static` member whose count places its parameters as if it took a receiver
        // answers `false` too.
        assert!(!MethodFacts::new("of", "(I)LH;", 2).has_receiver());
        assert!(
            !MethodFacts::new("value", "()I", 1)
                .with_access_flags(ACC_PUBLIC | ACC_STATIC)
                .has_receiver()
        );
    }

    #[test]
    fn parameter_types_places_an_arrays_successor_one_slot_past_the_array() {
        // JVMS 2.6.1, and the one place this fact can go wrong: an array of a `long` or a `double`
        // fills **one** slot, so the `int` after `[J` is at slot 1 and not at slot 2.
        let static_facts = MethodFacts::new("f", "(Z[JI)J", 3).with_access_flags(ACC_STATIC);
        assert_eq!(
            static_facts.parameter_types(),
            BTreeMap::from([
                (0, Type::Boolean),
                (1, Type::Reference("long[]".into())),
                (2, Type::Int)
            ])
        );

        // The same descriptor on an instance member: the receiver is slot 0 and every parameter
        // moves up one, which is the numbering `parameter_slots` counts too.
        let instance_facts = MethodFacts::new("f", "(Z[JI)J", 4).with_access_flags(0);
        assert_eq!(
            instance_facts.parameter_types(),
            BTreeMap::from([
                (1, Type::Boolean),
                (2, Type::Reference("long[]".into())),
                (3, Type::Int),
            ])
        );

        // A `double` fills two slots and a `double[][]` fills one — the same cell shape stated the
        // two ways it can be: the `long` after the array is one slot past the array and not two.
        assert_eq!(
            MethodFacts::new("g", "(D[[DI)J", 4)
                .with_access_flags(ACC_STATIC)
                .parameter_types(),
            BTreeMap::from([
                (0, Type::Double),
                (2, Type::Reference("double[][]".into())),
                (3, Type::Int),
            ])
        );

        // A caller that stated no flags states the layout the count places: 3 slots is the static
        // reading, 4 is the instance one, and a count that agrees with neither states nothing.
        assert_eq!(
            MethodFacts::new("f", "(Z[JI)J", 3).parameter_types(),
            static_facts.parameter_types()
        );
        assert_eq!(
            MethodFacts::new("f", "(Z[JI)J", 4).parameter_types(),
            instance_facts.parameter_types()
        );
        assert!(
            MethodFacts::new("f", "(Z[JI)J", 5)
                .parameter_types()
                .is_empty()
        );
        // A descriptor that is not a method descriptor states no type at all.
        assert!(
            MethodFacts::new("f", "not-a-descriptor", 1)
                .parameter_types()
                .is_empty()
        );
        assert!(MethodFacts::new("f", "([J", 1).parameter_types().is_empty());
    }
}
