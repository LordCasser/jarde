//! The order a constructor's synthetic pre-super stores are presented in.
//!
//! javac writes an inner, local or anonymous class's constructor with the synthetic captures
//! (`this$0` for the enclosing instance, `val$x` for a captured local) stored **before** the
//! constructor call — a sequence the JVM allows on the uninitialized `this`
//! (JVMS 4.10.1.9, which [`crate::field`] proves those writes under) but Java source does not:
//! a constructor's first statement must be the constructor call, and an assignment written
//! before it is a *flexible constructor body*, which javac reads as a preview feature and
//! refuses. Presenting the bytes in their own order ([`crate::build`]'s BCI order) is the right
//! presentation for the verifier and an uncompilable one for the source — this module is the
//! narrow normalization that turns the compiler's own pattern back into the source shape it
//! came from.
//!
//! # The judgement is a pattern plus a declaration fact, and it is all-or-nothing
//!
//! The statement order comes out of [`crate::build`] in BCI order, so the shape to look at is a
//! prefix of the body's top-level statements: every statement **before** the prologue
//! ([`crate::init`]) has to be one verified field write ([`crate::field`]) that satisfies
//!
//! * the written member is the class's own **synthetic** field — [`facts::ACC_SYNTHETIC`] on the
//!   field header the run read is the primary evidence, and the javac minting scheme
//!   (`this$<digits>`, a name starting with `val$`) is the fallback only for a run that read no
//!   field headers at all; a header that names the member and does **not** state the flag is a
//!   negative, however javac-shaped the name is, because a name alone is not the compiler's
//!   evidence;
//! * the receiver of the write is the frames' `UninitializedThis` — the constructor's own
//!   `this`, before its constructor call;
//! * the stored value is one parameter's direct load (`aload`/`iload` of a parameter slot), with
//!   no computation of its own.
//!
//! Every statement of the prefix has to satisfy the pattern: one that does not (a user field's
//! write, a computed value, an interleaved call) leaves the whole order as the bytes have it —
//! verbatim is the fallback, never a partial move. When the prefix is certified, the statements
//! are presented prologue-first: the `super(…)` call, then the group in its original order, then
//! the rest of the body in its original order. What moves is the *presentation*: each statement
//! keeps its own text, its own origins and its own records ([`crate::field`]'s verdicts are not
//! re-derived), and the JVM's own execution order is not a source-order claim — JLS 12.5 runs an
//! initializer's field writes after the constructor call, which is exactly the order the
//! normalized text has.
//!
//! # The move happens only past a constructor call that cannot observe the group's fields
//!
//! Placing the group after the constructor call is the source's order because a source
//! constructor call cannot read what it has not been handed. A class file's call is not bound by
//! that: a superclass constructor that virtually dispatches on `this` — the shape an anonymous
//! subclass overriding a hook the superclass constructor calls makes possible — runs the
//! subclass's own code *during* the call, and that code reads the captures from the very fields
//! this group writes, so where the bytes store them decides what it sees. The move is therefore
//! taken only past a call the group's fields cannot be observed through, and there are two
//! proofs of that, each one stated by this run's own facts:
//!
//! * **the target runs no user code**: `java/lang/Object`'s constructor is final and empty, so
//!   `java/lang/Object.<init>()V` — owner, name and descriptor exactly, as the prologue's own
//!   invoke states them — is the one target that provably dispatches nowhere;
//! * **the class declares no code the call could dispatch into**: a field can only be read by an
//!   instruction that names it, and the synthetic captures are minted into this class alone — no
//!   superclass was compiled against them — so the code that can read them during the call is
//!   this class's own code, reached through whatever virtual dispatch the superclass constructor
//!   makes. The class's method table (`build`'s `Inputs::class_methods`, the declaration view of
//!   the same class header [`is_synthetic_field`] reads the fields from) states every member this
//!   class declares: one that declares nothing but its constructors and its class initializer has
//!   no code that can run during the call — the other constructors are not the one running, and
//!   the class initializer ran before the instance existed (JLS 12.4.2). The call's arguments are
//!   read *before* it runs while the group moves after it, so this arm additionally walks the
//!   arguments' own SSA dependencies and refuses an argument built from a moved field, or one
//!   that invokes anything at all: a callee's body is not in this run's facts, so what it does is
//!   not a proof — and the one fact that would make such a call harmless (the verifier's own
//!   `uninitializedThis` rule, JVMS 4.10.1.9, which no callee can be handed the instance under)
//!   belongs to the frame pass, not to this rule's reading.
//!
//! Any other shape keeps the byte order, whose source shape javac refuses (a flexible constructor
//! body): an uncompilable text is a loud failure, where a recompiled program that reads `null`
//! where the original read a value is a silent one.
//!
//! [`facts::ACC_SYNTHETIC`]: crate::facts::ACC_SYNTHETIC

use std::collections::BTreeSet;

use jarde_jvm::method_ir::{Definition, SsaInstruction, SsaTable, Value, ValueId};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::MemberHeader;

use crate::ast::{AssignOp, ConstructorTarget, Stmt, StmtKind};
use crate::build::stack_operands;
use crate::decode::Operations;
use crate::facts::{ACC_SYNTHETIC, Operation};
use crate::field;
use crate::init;
use crate::stop::{self, StopReason};

/// Presents a constructor body with the prologue first when its pre-super statement prefix is the
/// compiler's certified synthetic-store group, and leaves it byte-verbatim when it is not.
///
/// `stmts` are the body's top-level statements in BCI order. Everything the criteria do not
/// prove — a body with no prologue, a `this(…)` prologue, a super prologue whose target is one
/// the group's fields can be observed through (see the module's own section on the move), an
/// empty prefix, a prefix holding anything but certified synthetic direct-parameter stores, or a
/// prologue statement the order does not place at top level — is a no-op that keeps the text
/// byte-verbatim.
pub(crate) fn present_prologue_first(
    stmts: &mut [Stmt],
    ssa: &SsaTable,
    operations: &Operations,
    fields: &field::Plan,
    class_fields: Option<&[MemberHeader]>,
    class_methods: Option<&[MemberHeader]>,
    declaring: Option<&str>,
    prologues: &init::Prologues,
    parameters: u16,
    has_receiver: bool,
    budget: &mut Budget,
) -> Result<(), StopReason> {
    let Some(prologue) = prologues.prologue() else {
        return Ok(());
    };
    // The normalized shape is a source one, and a source constructor delegates to its own class
    // through `this(…)` **as** the first statement, with nothing before it to move: only a
    // `super(…)` prologue has a pre-call prefix the compiler writes into the bytes.
    if prologue.target != ConstructorTarget::Super {
        return Ok(());
    }
    // The prologue is one statement of its own, and the group is what stands before it *there*:
    // a prologue the order did not write at top level (inside an `if`, inside a `try`) has no
    // prefix this presentation owns, and the verbatim order stays.
    let mut prologue_at = None;
    for (index, stmt) in stmts.iter().enumerate() {
        let is_prologue = matches!(
            stmt.kind,
            StmtKind::ConstructorCall {
                target: ConstructorTarget::Super,
                ..
            }
        ) && stmt.origin.primary().bci() == prologue.bci;
        if is_prologue {
            if prologue_at.is_some() {
                return Ok(());
            }
            prologue_at = Some(index);
        }
    }
    let Some(prologue_index) = prologue_at else {
        return Ok(());
    };
    // The group is the whole prefix: one statement that is not a certified synthetic
    // direct-parameter store — a user field's write, a computed value, anything else the
    // bytecode ran before the constructor call — is an interleaved effect this presentation
    // does not own, and the verbatim order stays. An empty prefix has nothing to move.
    if prologue_index == 0 {
        return Ok(());
    }
    // The members the group writes, as the same `field@1` verdicts the pattern check reads them:
    // what a constructor-call argument must not be built from, and what the second proof below
    // moves past a call it cannot be observed through.
    let mut moved: Vec<(&str, &str)> = Vec::with_capacity(prologue_index);
    for stmt in stmts[..prologue_index].iter() {
        stop::poll(budget, Some(stmt.origin.primary().bci()))?;
        stop::charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(stmt.origin.primary().bci()),
        )?;
        if !is_synthetic_parameter_store(
            stmt,
            ssa,
            operations,
            fields,
            class_fields,
            declaring,
            parameters,
            has_receiver,
        ) {
            return Ok(());
        }
        if let Some((evidence, _)) = fields.claim(stmt.origin.primary().bci()) {
            moved.push((evidence.name.as_str(), evidence.descriptor.as_str()));
        }
    }
    // Prologue first, then the group in its original order, then the rest in its original order:
    // the certified group runs up to the prologue, so this is one rotation of the prologue
    // statement to the front of that range. The rotation is gated on the constructor call being
    // one the group's fields cannot be observed through: the group is re-derived where JLS 12.5
    // runs field writes — *after* the call — and a superclass constructor that virtually
    // dispatches on `this` runs the subclass's own code during the call, reading these very
    // captures where the bytes store them. Two proofs of that, each from this run's own facts:
    //
    // * `java/lang/Object.<init>()V` is final and empty — the one target that provably runs no
    //   user code at all (owner, name and descriptor exactly as the prologue's invoke states
    //   them);
    // * the class declares no code the call could run: a field is only read by an instruction
    //   that names it, and the synthetic captures are minted into this class alone, so the code
    //   that can read them during the call is this class's own — reached through whatever
    //   virtual dispatch the superclass constructor makes. A method table that declares nothing
    //   but the constructors and the class initializer has no such code: the other constructors
    //   are not the one running, and the class initializer ran before the instance existed
    //   (JLS 12.4.2). The arguments are read *before* the call while the group moves after it,
    //   so that arm also requires every argument to be built without reading a moved field and
    //   without invoking anything (a callee's body is not in this run's facts, so the run cannot
    //   prove what it does).
    //
    // Any other target keeps the byte order, whose uncompilable source shape fails loudly where
    // the moved shape would silently change the program.
    let call_cannot_run_user_code = match operations.get(prologue.bci) {
        Some(Operation::Invoke(target)) => {
            (target.owner() == "java/lang/Object"
                && target.name() == "<init>"
                && target.descriptor() == "()V")
                || (target.name() == "<init>"
                    && declares_only_initializers(class_methods)
                    && call_arguments_avoid(
                        &moved,
                        ssa,
                        operations,
                        fields,
                        prologue.bci,
                        declaring,
                        budget,
                    )?)
        }
        _ => false,
    };
    if !call_cannot_run_user_code {
        return Ok(());
    }
    stmts[..=prologue_index].rotate_right(1);
    Ok(())
}

/// Whether one statement is a certified synthetic capture store: one `field@1` write on the
/// uninitialized `this` whose field is the class's own synthetic member and whose value is one
/// parameter's direct load.
fn is_synthetic_parameter_store(
    stmt: &Stmt,
    ssa: &SsaTable,
    operations: &Operations,
    fields: &field::Plan,
    class_fields: Option<&[MemberHeader]>,
    declaring: Option<&str>,
    parameters: u16,
    has_receiver: bool,
) -> bool {
    // The assignment one putfield writes; a compound update or an accessor's write is not this
    // shape, and neither is a field access the run did not present at this statement's anchor.
    let StmtKind::FieldAssign {
        receiver: Some(_),
        name,
        op: AssignOp::Assign,
        value: _,
    } = &stmt.kind
    else {
        return false;
    };
    let at = stmt.origin.primary().bci();
    let Some((evidence, shape)) = fields.claim(at) else {
        return false;
    };
    let Some(receiver) = shape.receiver else {
        return false;
    };
    let Some(stored) = shape.value else {
        return false;
    };
    // The pre-call write is on the constructor's own `this`, and the member it names is the class
    // being constructed — the two facts `field@1` proved the write under (JVMS 4.10.1.9).
    if evidence.is_static || !matches!(ssa.value(receiver).ty(), Value::UninitializedThis) {
        return false;
    }
    let Some(declaring) = declaring else {
        return false;
    };
    if declaring != evidence.owner || evidence.name != *name {
        return false;
    }
    is_synthetic_field(class_fields, &evidence.name, &evidence.descriptor)
        && is_direct_parameter_load(ssa, operations, stored, parameters, has_receiver)
}

/// Whether the class's own method table declares nothing the constructor call could run: no
/// member but its constructors and its class initializer.
///
/// A class file can only read a field through an instruction that names it, and the synthetic
/// captures are minted into the class being constructed alone — no superclass was compiled
/// against them — so the code that can read them during the constructor call is this class's own,
/// reached through whatever virtual dispatch the superclass constructor makes. A member other
/// than the constructor that is running is code this class declares, and the superclass's
/// constructor can reach it exactly that way; a method table that declares none is a class with
/// no code the call can run. The class initializer is not such a member: it runs before any
/// instance of the class exists (JLS 12.4.2), and an initializer's own run cannot be a
/// constructor call's.
///
/// The declaration view is the one the run's class header states (`build`'s
/// `Inputs::class_methods`), and a run that read no method table proves nothing here: it is a
/// negative, like a table that names a member this reading does not recognize.
fn declares_only_initializers(class_methods: Option<&[MemberHeader]>) -> bool {
    let Some(headers) = class_methods else {
        return false;
    };
    let mut names_a_constructor = false;
    for header in headers {
        let name = header.name.raw().0.as_slice();
        if name == b"<init>" {
            names_a_constructor = true;
        } else if name != b"<clinit>" {
            return false;
        }
    }
    names_a_constructor
}

/// Whether every argument of the constructor call at `bci` is a value built without reading one of
/// the `moved` fields and without running any code this run cannot see.
///
/// The call's arguments are evaluated *before* the call, and the group moves after it: an argument
/// built from a moved field would read the field where the bytes left it before the store, which
/// the moved text no longer writes — so the argument is refused wherever its value depends on one,
/// as a *dataflow* reading rather than the expression's spelling. The walk is over the SSA's own
/// definitions, so a value that reaches the field through locals, arithmetic or a cast is refused
/// the same way a direct read is; an invocation anywhere in that closure is refused too, because
/// the callee's body is not in this run's facts, so what it does is not a proof this run holds. An
/// unknown definition (a merge, a handler, an instruction this layer did not model) is a negative
/// as well: the walk proves only what it can read.
///
/// The receiver is not an argument: it is the constructor's own `this` — the value the group's
/// writes were proved on — and a call whose receiver is anything else is not the shape this
/// presentation owns.
fn call_arguments_avoid(
    moved: &[(&str, &str)],
    ssa: &SsaTable,
    operations: &Operations,
    fields: &field::Plan,
    bci: u32,
    declaring: Option<&str>,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    let Some(instruction) = instruction_at(ssa, bci) else {
        return Ok(false);
    };
    let operands = stack_operands(instruction);
    let Some((_, receiver)) = operands.first() else {
        return Ok(false);
    };
    if !matches!(ssa.value(*receiver).ty(), Value::UninitializedThis) {
        return Ok(false);
    }
    for (_, argument) in operands.iter().skip(1) {
        if value_observes_moved(*argument, moved, ssa, operations, fields, declaring, budget)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Whether one argument value is built, through any chain of the SSA's own definitions, from a
/// read of one of the `moved` fields, or from anything that runs code or states no value this
/// walk can read.
fn value_observes_moved(
    start: ValueId,
    moved: &[(&str, &str)],
    ssa: &SsaTable,
    operations: &Operations,
    fields: &field::Plan,
    declaring: Option<&str>,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    let mut pending = vec![start];
    let mut seen = BTreeSet::new();
    while let Some(value) = pending.pop() {
        if !seen.insert(value) {
            continue;
        }
        match ssa.value(value).def() {
            // The method's entry state: a parameter or `this`, the values the signature itself
            // hands the constructor. Nothing an entry value is can read a field.
            Definition::Entry { .. } => {}
            Definition::Instruction { bci, .. } => {
                // One step of this walk, charged like every other worklist step in the layer: the
                // seen set makes each value of the closure one visit.
                stop::charge(budget, CountedBudgetDimension::AnalysisSteps, 1, Some(*bci))?;
                let Some(operation) = operations.get(*bci) else {
                    return Ok(true);
                };
                if let Some((evidence, _)) = fields.claim(*bci) {
                    let names_a_moved = declaring == Some(evidence.owner.as_str())
                        && moved.iter().any(|(name, descriptor)| {
                            evidence.name == *name && evidence.descriptor == *descriptor
                        });
                    if names_a_moved {
                        return Ok(true);
                    }
                }
                // A call runs a body this run does not hold; the unmodelled remainder states no
                // value this walk can read; a monitor, a throw and a return are effects no
                // argument value is built from.
                if matches!(
                    operation,
                    Operation::Invoke(_)
                        | Operation::InvokeDynamic(_)
                        | Operation::Monitor { .. }
                        | Operation::Throw
                        | Operation::Return
                        | Operation::Other
                ) {
                    return Ok(true);
                }
                let Some(instruction) = instruction_at(ssa, *bci) else {
                    return Ok(true);
                };
                pending.extend(instruction.reads().iter().map(|(_, read)| *read));
            }
            // A merge or a handler is a definition this walk cannot follow to its operands: what
            // the value is there is not one chain of the body's own instructions.
            Definition::Phi { .. } | Definition::Caught { .. } => return Ok(true),
        }
    }
    Ok(false)
}

/// Whether the field one write names is the compiler's synthetic capture.
///
/// The field header the run read is the primary evidence — [`ACC_SYNTHETIC`] on the one header
/// that names this member. The javac minting scheme (`this$<digits>`, `val$…`) is the fallback
/// only where no header travelled, and a header that names the member without the flag is a
/// negative however the name is spelled.
fn is_synthetic_field(class_fields: Option<&[MemberHeader]>, name: &str, descriptor: &str) -> bool {
    match class_fields {
        Some(headers) => {
            let mut named = headers.iter().filter(|header| {
                header.name.raw().0.as_slice() == name.as_bytes()
                    && header.descriptor.raw().0.as_slice() == descriptor.as_bytes()
            });
            let (Some(header), None) = (named.next(), named.next()) else {
                // No header names this member, or two do: neither states the compiler's own
                // declaration, and the presentation claims nothing.
                return false;
            };
            header.access_flags & ACC_SYNTHETIC != 0
        }
        None => is_synthetic_name(name),
    }
}

/// Whether one field name is spelled the way javac mints synthetic captures.
fn is_synthetic_name(name: &str) -> bool {
    name.starts_with("val$")
        || name.strip_prefix("this$").is_some_and(|depth| {
            !depth.is_empty() && depth.bytes().all(|byte| byte.is_ascii_digit())
        })
}

/// Whether the stored value is one parameter's direct load — the value the constructor was
/// handed, with no computation of its own between the signature and the store.
fn is_direct_parameter_load(
    ssa: &SsaTable,
    operations: &Operations,
    stored: ValueId,
    parameters: u16,
    has_receiver: bool,
) -> bool {
    let Definition::Instruction { bci, .. } = ssa.value(stored).def() else {
        // A phi or a merge is a value the store's own argument flow does not state, and an
        // entry-defined value is not a stack value at all: neither is the direct pass.
        return false;
    };
    let Some(instruction) = instruction_at(ssa, *bci) else {
        return false;
    };
    let Some(Operation::Load { slot }) = operations.get(*bci) else {
        return false;
    };
    // The parameter slots are the signature's own: below `parameters`, and not the receiver —
    // which is `this` when the member has one and no parameter is.
    let receiver_slots = u16::from(has_receiver);
    *slot >= receiver_slots && *slot < parameters && {
        let writes = instruction.writes();
        writes.len() == 1 && writes[0].1 == stored
    }
}

/// The SSA record of one instruction, read from the table's own blocks.
fn instruction_at<'a>(ssa: &'a SsaTable, bci: u32) -> Option<&'a SsaInstruction> {
    ssa.blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| instruction.bci() == bci)
}
