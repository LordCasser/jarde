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
//! # The move happens only past a constructor call that cannot run user code
//!
//! Placing the group after the constructor call is the source's order because a source
//! constructor call cannot read what it has not been handed. A class file's call is not bound by
//! that: a superclass constructor that virtually dispatches on `this` — the shape an anonymous
//! subclass overriding a hook the superclass constructor calls makes possible — runs the
//! subclass's own code *during* the call, and that code reads the captures from the very fields
//! this group writes, so where the bytes store them decides what it sees. `java/lang/Object`'s
//! constructor is final and empty, so the one target that provably runs no user code is
//! `java/lang/Object.<init>()V`, as the prologue's own invoke states it — owner and descriptor
//! exactly. That is the only target the group moves past. Any other target keeps the byte order,
//! whose source shape javac refuses (a flexible constructor body): an uncompilable text is a
//! loud failure, where a recompiled program that reads `null` where the original read a value is
//! a silent one.
//!
//! [`facts::ACC_SYNTHETIC`]: crate::facts::ACC_SYNTHETIC

use jarde_jvm::method_ir::{Definition, SsaInstruction, SsaTable, Value, ValueId};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::MemberHeader;

use crate::ast::{AssignOp, ConstructorTarget, Stmt, StmtKind};
use crate::decode::Operations;
use crate::facts::{ACC_SYNTHETIC, Operation};
use crate::field;
use crate::init;
use crate::stop::{self, StopReason};

/// Presents a constructor body with the prologue first when its pre-super statement prefix is the
/// compiler's certified synthetic-store group, and leaves it byte-verbatim when it is not.
///
/// `stmts` are the body's top-level statements in BCI order. Everything the criteria do not
/// prove — a body with no prologue, a `this(…)` prologue, a super prologue whose target is not
/// `java/lang/Object`'s own `<init>()V`, an empty prefix, a prefix holding anything but
/// certified synthetic direct-parameter stores, or a prologue statement the order does not place
/// at top level — is a no-op that keeps the text byte-verbatim.
pub(crate) fn present_prologue_first(
    stmts: &mut [Stmt],
    ssa: &SsaTable,
    operations: &Operations,
    fields: &field::Plan,
    class_fields: Option<&[MemberHeader]>,
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
    }
    // Prologue first, then the group in its original order, then the rest in its original order:
    // the certified group runs up to the prologue, so this is one rotation of the prologue
    // statement to the front of that range. The rotation is gated on the constructor call being
    // one no user code can run in: the group is re-derived where JLS 12.5 runs field writes —
    // *after* the call — and a superclass constructor that virtually dispatches on `this` runs
    // the subclass's own code during the call, reading these very captures where the bytes store
    // them. `java/lang/Object.<init>()V` is final and empty — the one target that cannot — and
    // any other target keeps the byte order, whose uncompilable source shape fails loudly where
    // the moved shape would silently change the program.
    let call_cannot_run_user_code = match operations.get(prologue.bci) {
        Some(Operation::Invoke(target)) => {
            target.owner() == "java/lang/Object"
                && target.name() == "<init>"
                && target.descriptor() == "()V"
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
