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

use std::collections::{BTreeMap, BTreeSet};

use jarde_jvm::method_ir::{RefType, SsaInstruction, SsaTable, Value, ValueId};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::MemberHeader;
use serde::Serialize;

use crate::ast::{AssignOp, Expr, ExprKind, Stmt, StmtKind};
use crate::build::stack_operands;
use crate::decode::Operations;
use crate::evidence::Publication;
use crate::facts::{DeclaringClass, FieldAccess, Operation, internal_form};
use crate::names::is_java_identifier;
use crate::pass::{FIELD, Precondition, RuleVersion};
use crate::refusal::{Gap, Refusal};
use crate::stop::{self, StopReason};

/// The pass answerable for every verdict of this module.
pub(crate) const RULE: RuleVersion = FIELD.rule();

/// The requirement a write on an uninitialized `this` states.
const DECLARING_CLASS: Precondition = Precondition::Metadata {
    attribute: "declaring_class",
};

/// Every field instruction of one body this rule read, the ones it claimed, and the gaps it states
/// in every selection.
///
/// The plan holds the **decisions**: what each claimed instruction reads (which the builder writes)
/// and every refusal with the evidence it refused. [`Self::materialize`] writes the owning
/// [`FieldRecord`]s from them after the artifact is committed, one record per charge and only when
/// the request selected `RuleDetails`.
pub(crate) struct Plan {
    claimed: BTreeMap<u32, (Evidence, Shape)>,
    /// Why a field instruction was not presented, in BCI order — the internal decision, with the
    /// evidence the record states. A gap is not the optional evidence: it is what every selection
    /// reports about an instruction this rule refused.
    refusals: Vec<(Evidence, Refusal)>,
}

/// One verdict of this rule's plan: the access it claimed, or the instruction it refused.
enum Decision<'a> {
    Claimed(&'a Evidence, bool),
    Refused(&'a Evidence, &'a Refusal),
}

impl Decision<'_> {
    fn at(&self) -> u32 {
        match self {
            Self::Claimed(evidence, _) | Self::Refused(evidence, _) => evidence.bci,
        }
    }

    /// The owning record, built here and only here.
    fn record(self) -> FieldRecord {
        crate::demand_counts::record_built(crate::evidence::RecoveryEvidenceKind::RuleDetails);
        match self {
            Self::Claimed(evidence, true) => FieldRecord::of_presented(evidence),
            Self::Claimed(evidence, false) => FieldRecord::of(
                evidence,
                false,
                Some(FieldRefusal::not_emitted(evidence.bci)),
            ),
            Self::Refused(evidence, refusal) => FieldRecord::of(
                evidence,
                false,
                Some(FieldRefusal::of(refusal, evidence.bci)),
            ),
        }
    }
}

impl Plan {
    /// An empty plan: a body that reads and writes no field at all.
    pub(crate) fn empty() -> Self {
        Self {
            claimed: BTreeMap::new(),
            refusals: Vec::new(),
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

    /// Whether a claimed write is proven safe to spell with the current class's simple field name.
    pub(crate) fn simple_static_final_write(&self, bci: u32) -> bool {
        self.claimed
            .get(&bci)
            .is_some_and(|(_, shape)| shape.writes() && shape.simple_static_final)
    }

    /// The field names that the naming walk must reserve for this body's proven simple writes.
    pub(crate) fn simple_static_final_names(
        &self,
        budget: &mut Budget,
    ) -> Result<BTreeSet<String>, StopReason> {
        stop::poll(budget, None)?;
        let mut names = BTreeSet::new();
        for (evidence, shape) in self.claimed.values() {
            stop::charge(
                budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(evidence.bci),
            )?;
            if shape.writes() && shape.simple_static_final {
                names.insert(evidence.name.clone());
            }
        }
        Ok(names)
    }

    /// Every field instruction the rule read and did not present, in BCI order.
    pub(crate) fn refusals<'a>(
        &'a self,
        presented: &'a BTreeSet<u32>,
    ) -> impl Iterator<Item = Gap> + 'a {
        let mut plan_refusals = self.refusals.iter().peekable();
        let mut not_emitted = self
            .claimed
            .keys()
            .filter(|at| !presented.contains(at))
            .peekable();
        std::iter::from_fn(move || {
            let take_plan = match (plan_refusals.peek(), not_emitted.peek()) {
                (Some((evidence, _)), Some(at)) => evidence.bci < **at,
                (Some(_), None) => true,
                (None, Some(_)) => false,
                (None, None) => return None,
            };
            if take_plan {
                let (evidence, refusal) = plan_refusals.next()?;
                let refusal = FieldRefusal::of(refusal, evidence.bci);
                Some(Gap::at(refusal.code, evidence.bci, refusal.message))
            } else {
                let at = *not_emitted.next()?;
                let refusal = FieldRefusal::not_emitted(at);
                Some(Gap::at(refusal.code, at, refusal.message))
            }
        })
    }

    /// Whether this rule decided anything about this body.
    pub(crate) fn answered(&self) -> bool {
        !self.claimed.is_empty() || !self.refusals.is_empty()
    }

    /// The owning records this plan publishes under `publication`, in BCI order, within the phase's
    /// remaining allowance.
    pub(crate) fn materialize(
        &self,
        presented: &BTreeSet<u32>,
        publication: Publication,
        phase: &mut crate::evidence::EvidencePhase,
        budget: &mut jarde_reader::budget::Budget,
    ) -> (Vec<FieldRecord>, crate::evidence::Materialized) {
        let mut decisions: Vec<Decision<'_>> = self
            .claimed
            .values()
            .map(|(evidence, _)| Decision::Claimed(evidence, presented.contains(&evidence.bci)))
            .chain(
                self.refusals
                    .iter()
                    .map(|(evidence, refusal)| Decision::Refused(evidence, refusal)),
            )
            .collect();
        decisions.sort_by_key(Decision::at);
        phase.materialize(
            budget,
            decisions
                .into_iter()
                .filter(|decision| publication.publishes(&[decision.at()])),
            Decision::record,
        )
    }

    /// How many field instructions the rule read, and how many of them it presented. The two counts
    /// are the rule's own work over the whole body, so the report states them whatever the caller
    /// selected and whatever range the selection carried.
    pub(crate) fn counts(&self, receipt: &BTreeSet<u32>) -> (u64, u64) {
        let claimed = u64::try_from(self.claimed.len()).unwrap_or(u64::MAX);
        let presented = u64::try_from(
            self.claimed
                .keys()
                .filter(|at| receipt.contains(at))
                .count(),
        )
        .unwrap_or(u64::MAX);
        let refused = u64::try_from(self.refusals.len()).unwrap_or(u64::MAX);
        (claimed.saturating_add(refused), presented)
    }
}

/// Collects only real field operations in the final AST. Direct origins and the plan must agree;
/// derived origins, bytecode quotes and synthetic accessor field text cannot create a receipt.
pub(crate) fn committed_presentations(
    program: &crate::build::Program,
    plan: &Plan,
    budget: &mut Budget,
) -> Result<BTreeSet<u32>, StopReason> {
    enum Node<'a> {
        Stmt(&'a Stmt),
        Expr(&'a Expr),
    }
    let mut pending: Vec<_> = program.stmts.iter().map(Node::Stmt).collect();
    let mut presented = BTreeSet::new();
    while let Some(node) = pending.pop() {
        let at = match node {
            Node::Stmt(stmt) => stmt.origin.primary().bci(),
            Node::Expr(expr) => expr.origin.primary().bci(),
        };
        stop::charge(budget, CountedBudgetDimension::AnalysisSteps, 1, Some(at))?;
        stop::charge(budget, CountedBudgetDimension::IrItems, 1, Some(at))?;
        let mut record = |at: u32, access: FieldAccess, name: &str| {
            if plan
                .claim(at)
                .is_some_and(|(evidence, _)| evidence.access == access && evidence.name == name)
            {
                presented.insert(at);
            }
        };
        match node {
            Node::Stmt(stmt) => match &stmt.kind {
                StmtKind::Declare { value, .. } | StmtKind::Return { value } => {
                    if let Some(value) = value {
                        pending.push(Node::Expr(value));
                    }
                    if let StmtKind::Return { value: Some(value) } = &stmt.kind
                        && stmt.origin.primary().method().is_none()
                        && let Some(shape) = program.field_increments.get(&at)
                    {
                        let mut update = value;
                        while let ExprKind::Cast { value, .. } = &update.kind {
                            update = value;
                        }
                        if matches!(&update.kind, ExprKind::Local(_))
                            && update.origin.primary().method().is_none()
                            && update.origin.primary().bci() == shape.write
                            && update.origin.derived().iter().any(|origin| {
                                origin.method().is_none() && origin.bci() == shape.read
                            })
                        {
                            if let Some((read, _)) = plan.claim(shape.read) {
                                record(shape.read, FieldAccess::Read, &read.name);
                            }
                            if let Some((write, _)) = plan.claim(shape.write) {
                                record(shape.write, FieldAccess::Write, &write.name);
                            }
                        }
                    }
                }
                StmtKind::Assign { value, .. }
                | StmtKind::Expr(value)
                | StmtKind::Throw { value } => pending.push(Node::Expr(value)),
                StmtKind::FieldAssign {
                    receiver,
                    name,
                    op,
                    value,
                } => {
                    if stmt.origin.primary().method().is_none() {
                        record(at, FieldAccess::Write, name);
                    }
                    if *op == AssignOp::Add {
                        for origin in stmt.origin.derived() {
                            stop::charge(
                                budget,
                                CountedBudgetDimension::IrItems,
                                1,
                                Some(origin.bci()),
                            )?;
                            if origin.method().is_none() {
                                record(origin.bci(), FieldAccess::Read, name);
                            }
                        }
                    }
                    if let Some(receiver) = receiver {
                        pending.push(Node::Expr(receiver));
                    }
                    pending.push(Node::Expr(value));
                }
                StmtKind::IndexAssign {
                    array,
                    index,
                    value,
                    ..
                } => {
                    pending.extend([Node::Expr(array), Node::Expr(index), Node::Expr(value)]);
                }
                StmtKind::ConstructorCall { args, .. } => {
                    pending.extend(args.iter().map(Node::Expr))
                }
                StmtKind::If {
                    cond,
                    then_body,
                    else_body,
                } => {
                    pending.push(Node::Expr(cond));
                    pending.extend(then_body.iter().map(Node::Stmt));
                    pending.extend(else_body.iter().map(Node::Stmt));
                }
                StmtKind::While { cond, body, .. } | StmtKind::DoWhile { cond, body, .. } => {
                    pending.push(Node::Expr(cond));
                    pending.extend(body.iter().map(Node::Stmt));
                }
                StmtKind::For {
                    init,
                    cond,
                    update,
                    body,
                    ..
                } => {
                    pending.extend([Node::Stmt(init), Node::Expr(cond), Node::Stmt(update)]);
                    pending.extend(body.iter().map(Node::Stmt));
                }
                StmtKind::ForEach { iterable, body, .. } => {
                    pending.push(Node::Expr(iterable));
                    pending.extend(body.iter().map(Node::Stmt));
                }
                StmtKind::Switch { value, arms } => {
                    pending.push(Node::Expr(value));
                    for arm in arms {
                        pending.extend(arm.body.iter().map(Node::Stmt));
                    }
                }
                StmtKind::Try {
                    resources,
                    catches,
                    body,
                    finally_body,
                } => {
                    pending.extend(resources.iter().map(|resource| Node::Expr(&resource.value)));
                    for clause in catches {
                        pending.extend(clause.body.iter().map(Node::Stmt));
                    }
                    pending.extend(body.iter().map(Node::Stmt));
                    if let Some(body) = finally_body {
                        pending.extend(body.iter().map(Node::Stmt));
                    }
                }
                StmtKind::Synchronized { lock, body } => {
                    pending.push(Node::Expr(lock));
                    pending.extend(body.iter().map(Node::Stmt));
                }
                StmtKind::Break { .. } | StmtKind::Continue { .. } | StmtKind::Fallback { .. } => {}
            },
            Node::Expr(expr) => match &expr.kind {
                ExprKind::Field { receiver, name } => {
                    if expr.origin.primary().method().is_none() {
                        record(at, FieldAccess::Read, name);
                    }
                    pending.push(Node::Expr(receiver));
                }
                ExprKind::PostIncrement { target } => {
                    if let ExprKind::Field { name, .. } = &target.kind
                        && expr.origin.primary().method().is_none()
                    {
                        record(at, FieldAccess::Write, name);
                    }
                    pending.push(Node::Expr(target));
                }
                ExprKind::Call { receiver, args, .. } => {
                    if let Some(receiver) = receiver {
                        pending.push(Node::Expr(receiver));
                    }
                    pending.extend(args.iter().map(Node::Expr));
                }
                ExprKind::New {
                    qualifier, args, ..
                } => {
                    if let Some(qualifier) = qualifier {
                        pending.push(Node::Expr(qualifier));
                    }
                    pending.extend(args.iter().map(Node::Expr));
                }
                ExprKind::Lambda { body, .. }
                | ExprKind::MethodReference {
                    qualifier: body, ..
                }
                | ExprKind::InstanceOf { value: body, .. }
                | ExprKind::ArrayLength { array: body }
                | ExprKind::Cast { value: body, .. }
                | ExprKind::Not { value: body }
                | ExprKind::Neg { value: body } => pending.push(Node::Expr(body)),
                ExprKind::Index { array, index }
                | ExprKind::Binary {
                    left: array,
                    right: index,
                    ..
                } => {
                    pending.extend([Node::Expr(array), Node::Expr(index)]);
                }
                ExprKind::NewArray {
                    lengths,
                    initializers,
                    ..
                } => {
                    pending.extend(lengths.iter().map(Node::Expr));
                    if let Some(values) = initializers {
                        pending.extend(values.iter().map(Node::Expr));
                    }
                }
                ExprKind::Conditional {
                    test,
                    when_true,
                    when_false,
                } => {
                    pending.extend([
                        Node::Expr(test),
                        Node::Expr(when_true),
                        Node::Expr(when_false),
                    ]);
                }
                ExprKind::Concat { parts } => {
                    pending.extend(parts.iter().map(|part| Node::Expr(&part.value)))
                }
                ExprKind::Local(_)
                | ExprKind::Integer(_)
                | ExprKind::Boolean(_)
                | ExprKind::Long(_)
                | ExprKind::Float(_)
                | ExprKind::Double(_)
                | ExprKind::Str(_)
                | ExprKind::Null
                | ExprKind::ClassLiteral { .. }
                | ExprKind::Path(_)
                | ExprKind::Super { .. } => {}
            },
        }
    }
    Ok(presented)
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
    /// Whether this claimed write has the same-class blank static-final declaration proof.
    simple_static_final: bool,
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
///
/// Every instruction is read and decided for every selection; no owning record is built here. The
/// verdicts stay in the [`Plan`] and [`Plan::materialize`] writes the records from them after the
/// artifact is committed.
pub(crate) fn plan(
    ssa: &SsaTable,
    operations: &Operations,
    declaring: Option<&DeclaringClass>,
    method_name: &str,
    method_descriptor: &str,
    class_fields: Option<&[MemberHeader]>,
    budget: &mut Budget,
) -> Result<Plan, StopReason> {
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
                plan.claimed.insert(at, (evidence, shape));
            }
            Err(refusal) => plan.refusals.push((evidence, refusal)),
        }
    }
    plan.refusals.sort_by_key(|(evidence, _)| evidence.bci);
    prove_simple_static_final(
        &mut plan,
        declaring,
        method_name,
        method_descriptor,
        class_fields,
        budget,
    )?;
    Ok(plan)
}

/// Proves the narrow blank-`static final` spelling rule from the field plan's own claims and the
/// already-read headers of the declaring class. A same-name field with another descriptor is still
/// ambiguous in Java, so uniqueness is checked by name before the descriptor and flags.
fn prove_simple_static_final(
    plan: &mut Plan,
    declaring: Option<&DeclaringClass>,
    method_name: &str,
    method_descriptor: &str,
    class_fields: Option<&[MemberHeader]>,
    budget: &mut Budget,
) -> Result<(), StopReason> {
    const ACC_FINAL: u16 = 0x0010;
    const ACC_ENUM: u16 = 0x4000;

    stop::poll(budget, None)?;
    if method_name != "<clinit>" || method_descriptor != "()V" {
        return Ok(());
    }
    let Some(declaring) = declaring else {
        return Ok(());
    };
    if declaring.is_interface() || declaring.access_flags() & ACC_ENUM != 0 {
        return Ok(());
    }
    let Some(class_fields) = class_fields else {
        return Ok(());
    };

    // Index the already-read field headers once. `None` records a same-name ambiguity, including
    // an otherwise matching descriptor, so the proof never becomes a name-plus-descriptor guess.
    let mut declarations: BTreeMap<&[u8], Option<&MemberHeader>> = BTreeMap::new();
    for field in class_fields {
        stop::charge(budget, CountedBudgetDimension::IrItems, 1, None)?;
        let name = field.name.raw().0.as_slice();
        match declarations.entry(name) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(Some(field));
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                entry.insert(None);
            }
        }
    }

    for (evidence, shape) in plan.claimed.values_mut() {
        stop::charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(evidence.bci),
        )?;
        if !evidence.is_static
            || !shape.writes()
            || evidence.owner != declaring.name()
            || !is_java_identifier(&evidence.name)
        {
            continue;
        }

        let Some(Some(field)) = declarations.get(evidence.name.as_bytes()) else {
            continue;
        };
        if field.descriptor.raw().0.as_slice() != evidence.descriptor.as_bytes()
            || field.access_flags & (crate::facts::ACC_STATIC | ACC_FINAL)
                != (crate::facts::ACC_STATIC | ACC_FINAL)
        {
            continue;
        }
        let mut has_constant_value = false;
        for attribute in &field.attributes {
            stop::charge(
                budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(evidence.bci),
            )?;
            has_constant_value |= attribute.name.raw().0.as_slice() == b"ConstantValue";
        }
        if has_constant_value {
            continue;
        }
        shape.simple_static_final = true;
    }
    Ok(())
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
            simple_static_final: false,
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
        simple_static_final: false,
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
    fn not_emitted(bci: u32) -> Self {
        Self {
            code: "jre_field_not_emitted",
            rule: RULE,
            requirement: None,
            message: format!(
                "the field access at BCI {bci} passed field identity proof, but no matching Java field operation was emitted by the final body"
            ),
        }
    }
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
    use crate::ast::AssignOp;
    use crate::build::Program;
    use crate::pass::IrTable;
    use crate::source_map::{Origin, OriginSet};
    use jarde_reader::budget::{CancellationToken, Limits};

    #[test]
    fn final_refusal_gaps_keep_bci_order_across_plan_and_emission() {
        let evidence = |bci| Evidence {
            bci,
            access: FieldAccess::Read,
            is_static: true,
            owner: "Example".to_owned(),
            name: "value".to_owned(),
            descriptor: "I".to_owned(),
        };
        let mut plan = Plan::empty();
        plan.refusals.push((
            evidence(2),
            Refusal::shape("jre_field_shape", "unproved".to_owned()),
        ));
        plan.refusals.push((
            evidence(8),
            Refusal::shape("jre_field_shape", "unproved".to_owned()),
        ));
        plan.claimed.insert(
            5,
            (
                evidence(5),
                Shape {
                    receiver: None,
                    value: None,
                    simple_static_final: false,
                },
            ),
        );
        let gaps = plan
            .refusals(&BTreeSet::new())
            .map(|gap| gap.message().to_owned())
            .collect::<Vec<_>>();
        for (gap, bci) in gaps.iter().zip([2, 5, 8]) {
            assert!(gap.contains(&format!("BCI {bci}")), "{gaps:?}");
        }
    }

    #[test]
    fn receipt_budget_and_cancellation_never_return_a_partial_verdict() {
        let mut plan = Plan::empty();
        plan.claimed.insert(
            7,
            (
                Evidence {
                    bci: 7,
                    access: FieldAccess::Write,
                    is_static: true,
                    owner: "Example".to_owned(),
                    name: "value".to_owned(),
                    descriptor: "I".to_owned(),
                },
                Shape {
                    receiver: None,
                    value: None,
                    simple_static_final: false,
                },
            ),
        );
        let program = Program {
            stmts: vec![Stmt::new(
                StmtKind::FieldAssign {
                    receiver: None,
                    name: "value".to_owned(),
                    op: AssignOp::Assign,
                    value: Expr::direct(ExprKind::Integer(1), 6),
                },
                OriginSet::new(Origin::direct(7)),
            )],
            field_increments: BTreeMap::new(),
            statements: 1,
            ragged: false,
            lambdas: Vec::new(),
            accessors: Vec::new(),
            array_constructor_sites: Vec::new(),
            lambda_refusals: Vec::new(),
            accessor_refusals: Vec::new(),
            lambdas_presented: 0,
            accessors_presented: 0,
        };
        let limits = Limits {
            analysis_steps: 1,
            ir_items: u64::MAX,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        };
        let mut limited = Budget::new(limits.clone());
        assert!(matches!(
            committed_presentations(&program, &plan, &mut limited),
            Err(StopReason::Budget {
                dimension: CountedBudgetDimension::AnalysisSteps,
                ..
            })
        ));

        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(limits, token);
        assert!(matches!(
            committed_presentations(&program, &plan, &mut cancelled),
            Err(StopReason::Cancelled { .. })
        ));

        let mut complete = Budget::new(Limits {
            analysis_steps: u64::MAX,
            ir_items: u64::MAX,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        assert_eq!(
            committed_presentations(&program, &plan, &mut complete).expect("complete receipt"),
            BTreeSet::from([7])
        );
    }

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

    #[test]
    fn simple_static_final_proof_charges_claimed_work() {
        let mut plan = Plan::empty();
        plan.claimed.insert(
            7,
            (
                Evidence {
                    bci: 7,
                    access: FieldAccess::Write,
                    is_static: true,
                    owner: "FinalStaticProbe".to_owned(),
                    name: "first".to_owned(),
                    descriptor: "I".to_owned(),
                },
                Shape {
                    receiver: None,
                    value: None,
                    simple_static_final: false,
                },
            ),
        );
        let declaring = DeclaringClass::new("FinalStaticProbe", 0);
        let mut budget = Budget::new(Limits {
            ir_items: 0,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        assert!(matches!(
            prove_simple_static_final(
                &mut plan,
                Some(&declaring),
                "<clinit>",
                "()V",
                Some(&[]),
                &mut budget,
            ),
            Err(StopReason::Budget {
                dimension: CountedBudgetDimension::IrItems,
                at: Some(7),
                ..
            })
        ));
    }

    #[test]
    fn simple_static_final_proof_honors_cancellation_before_indexing() {
        let token = CancellationToken::new();
        token.cancel();
        let mut budget = Budget::with_cancellation_token(
            Limits {
                elapsed_millis: u64::MAX,
                ..Limits::default()
            },
            token,
        );
        let mut plan = Plan::empty();
        let declaring = DeclaringClass::new("FinalStaticProbe", 0);
        assert!(matches!(
            prove_simple_static_final(
                &mut plan,
                Some(&declaring),
                "<clinit>",
                "()V",
                Some(&[]),
                &mut budget,
            ),
            Err(StopReason::Cancelled { at: None })
        ));

        let mut names_budget = Budget::with_cancellation_token(
            Limits {
                elapsed_millis: u64::MAX,
                ..Limits::default()
            },
            {
                let token = CancellationToken::new();
                token.cancel();
                token
            },
        );
        assert!(matches!(
            plan.simple_static_final_names(&mut names_budget),
            Err(StopReason::Cancelled { at: None })
        ));
    }
}
