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

/// The access-flag bit a class sets on a member that is `static`.
pub const ACC_STATIC: u16 = 0x0008;

/// The access-flag bit a class sets on the bridge method a compiler generated.
pub const ACC_BRIDGE: u16 = 0x0040;

/// The access-flag bit a class sets on a member a compiler generated.
pub const ACC_SYNTHETIC: u16 = 0x1000;

/// What a class file says about the method whose body is being presented.
///
/// Identity: the name and descriptor are evidence the presentation quotes, and the parameter
/// count is what tells an ordinal name whether it belongs to a parameter or to a local.
///
/// The member's **access flags** are here too, and they are optional because the payload does not
/// carry them (P3 1.3b publishes the body's own tables, not the member list): a caller that read
/// the class's declaration states them, and one that did not states nothing. The difference
/// matters to exactly one rule — a **bridge** is a member the compiler declared as one
/// ([`ACC_BRIDGE`]), so a run that states no flags has no bridge to present and says so by having
/// no verdict to record, rather than by guessing from the body's shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodFacts {
    name: String,
    descriptor: String,
    parameters: u16,
    access_flags: Option<u16>,
}

impl MethodFacts {
    /// One method's identity: its name, its descriptor and its number of parameter slots.
    pub fn new(name: impl Into<String>, descriptor: impl Into<String>, parameters: u16) -> Self {
        Self {
            name: name.into(),
            descriptor: descriptor.into(),
            parameters,
            access_flags: None,
        }
    }

    /// The same facts with the member's access flags, exactly as the class declared them.
    pub fn with_access_flags(mut self, access_flags: u16) -> Self {
        self.access_flags = Some(access_flags);
        self
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvokeKind {
    Static,
    Virtual,
    Special,
    Interface,
}

/// One symbolic reference an invocation names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallTarget {
    kind: InvokeKind,
    owner: String,
    name: String,
    descriptor: String,
}

impl CallTarget {
    /// One invocation's target: how it is reached, its owner as the class file spells it (internal
    /// form, `java/lang/Object`), its name and its descriptor.
    pub fn new(
        kind: InvokeKind,
        owner: impl Into<String>,
        name: impl Into<String>,
        descriptor: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            owner: owner.into(),
            name: name.into(),
            descriptor: descriptor.into(),
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
    /// A dynamic call site: it reads the captured values the site's descriptor names and produces
    /// the instance the descriptor returns.
    ///
    /// The **shape** is not here. Whether this site is a lambda, a method reference or something
    /// this layer must not touch is decided from the class's bootstrap table, the factory method
    /// handle and the implementation handle ([`crate::lambda`], rule `lambda@1`), because a site
    /// with an arbitrary bootstrap must never be presented as one of them (A04).
    InvokeDynamic(DynamicSite),
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

/// One member of the class the presented body belongs to, with its declaration and its body.
///
/// This is the evidence the `accessor@1` rule needs and the payload cannot hold: a synthetic
/// accessor is a **different member** of the same class, so the run that presents the caller's body
/// can only see it if the caller hands it over. What the caller hands over is not a verdict — it
/// is the same two things the payload holds for the presented body (the member's declaration facts
/// and its decoded body), and every judgement about that body is made in [`crate::accessor`].
#[derive(Clone, Debug)]
pub struct MemberBody {
    owner: String,
    name: String,
    descriptor: String,
    access_flags: u16,
    code: jarde_reader::classfile::MethodCodeFacts,
}

impl MemberBody {
    /// One member of a class, exactly as the same read of that class stated it.
    pub fn new(
        owner: impl Into<String>,
        name: impl Into<String>,
        descriptor: impl Into<String>,
        access_flags: u16,
        code: jarde_reader::classfile::MethodCodeFacts,
    ) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
            descriptor: descriptor.into(),
            access_flags,
            code,
        }
    }

    /// The class the member is declared in, in internal form.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// The member's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The member's descriptor.
    pub fn descriptor(&self) -> &str {
        &self.descriptor
    }

    /// The member's access flags, as the class declared them.
    pub fn access_flags(&self) -> u16 {
        self.access_flags
    }

    /// The member's decoded body.
    pub fn code(&self) -> &jarde_reader::classfile::MethodCodeFacts {
        &self.code
    }
}

/// The members of one class, as the caller read them beside the presented body.
///
/// The caller that reads a body's facts has the class's declaration in hand (that is where its own
/// member name came from); handing the *other* members over is what makes a call to a synthetic
/// accessor decidable at all. The seam keeps the discipline of the payload: a member arrives as
/// its declaration and its **decoded body**, never as a claim about what it does, and
/// [`crate::accessor`] is the only place that reads a meaning into it.
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
    debug_locals: Vec<Option<String>>,
}

impl RecoveryFacts {
    /// The facts of one method, with no debug names.
    pub fn new(method: MethodFacts) -> Self {
        Self {
            method,
            debug_locals: Vec::new(),
        }
    }

    /// The same facts with one raw debug name per local slot, in slot order.
    ///
    /// A slot the body has no name for is `None`, and a body compiled without debug metadata passes
    /// an empty list: both mean "no evidence", and [`crate::names`] then derives the ordinal name.
    pub fn with_debug_locals(mut self, debug_locals: Vec<Option<String>>) -> Self {
        self.debug_locals = debug_locals;
        self
    }

    /// The method these facts describe.
    pub fn method(&self) -> &MethodFacts {
        &self.method
    }

    /// The raw name of each local slot, in slot order; empty when the body carries none.
    pub fn debug_locals(&self) -> &[Option<String>] {
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
        let facts = RecoveryFacts::new(MethodFacts::new("add", "(II)I", 3))
            .with_debug_locals(vec![Some("this".into()), Some("left".into())]);
        assert_eq!(facts.method().name(), "add");
        assert_eq!(facts.method().descriptor(), "(II)I");
        assert_eq!(facts.method().parameters(), 3);
        assert_eq!(facts.debug_locals().len(), 2);
        assert_eq!(facts.debug_locals()[1].as_deref(), Some("left"));
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
}
