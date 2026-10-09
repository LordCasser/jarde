//! javac's `assert` lowering, recognized as the three-part synthetic pattern it is.
//!
//! An `assert` statement compiles to three artifacts no single body holds: the synthetic
//! `static final boolean $assertionsDisabled` field (name plus `ACC_SYNTHETIC`), its `<clinit>`
//! initialization `X.class.desiredAssertionStatus() ? 0 : 1` (X the outermost enclosing class,
//! which is why the class literal must be proved and not assumed), and — at each use site — the
//! guard `if (!X.$assertionsDisabled) { if (!cond) throw new AssertionError(...); }` with the
//! message evaluated only on the failure path. This module reads that pattern off **already
//! recovered** statements: it never decodes bytecode and never decides a region, because the
//! pattern's parts are members of one class and the presentation of each member is the build
//! layer's own verdict. The class-source assembly that owns the class-wide census drives the
//! fold ([`crate::report`]'s `class_source_assert_*` entries); what lives here is the shape of
//! each half.
//!
//! # Why recognition refuses rather than approximates
//!
//! Every condition of the fold is structural, and an unmet one leaves the member exactly as the
//! recovery wrote it. A guard whose inner branch nests (`assert a || b` lowers to two nested
//! tests), a message the build could not present, or a read of the switch field outside a guard
//! are all shapes javac did not lower an `assert` to *in this form* — rewriting them would state
//! a program the class file does not have, which is the one thing this layer must not do. The
//! wrong-owner hazard is the sharpest of these: a hand-patched `<clinit>` that reads another
//! class's `desiredAssertionStatus()` still has a perfectly-shaped guard, and folding it would
//! silently move the switch to the current class — which is why the caller proves the
//! initialization line's class literal against the class's own nesting before any fold runs.

use crate::ast::{
    AssignOp, BinaryOp, CatchClause, Expr, ExprKind, Stmt, StmtKind, SwitchArm, Type,
};
use crate::stop;
use jarde_reader::budget::{Budget, CountedBudgetDimension};

/// The field name javac's `Lower` gives the assert switch, spelled exactly as the member table
/// spells it.
pub(crate) const SWITCH_FIELD_NAME: &str = "$assertionsDisabled";

/// The type name of the error construction an `assert` failure throws.
const ASSERTION_ERROR: &str = "java.lang.AssertionError";

/// One member's guard sites folded to `assert` statements, with the reads the fold consumed.
///
/// The reads are the BCIs of the guards' own `getstatic` instructions: the class-wide census the
/// caller holds names every physical read of the switch field, and the fold is only sound as a
/// whole — a read the fold did not consume would reference a field the caller is about to hide,
/// so the caller checks these against its census and keeps the member verbatim on any mismatch.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssertMemberFold {
    /// The member's statements with every foldable guard rewritten to `assert`.
    pub statements: Vec<Stmt>,
    /// The `getstatic` BCI of each guard this fold rewrote, in statement order.
    pub reads: Vec<u32>,
}

/// The proved switch-initialization line of one `<clinit>` body.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssertSwitchLine {
    /// The source spelling of the class whose `desiredAssertionStatus()` the line reads.
    pub class_literal: String,
    /// The `putstatic` BCI of the line's field write.
    pub write_bci: u32,
    /// The statement's position in the `<clinit>` program.
    pub position: usize,
}

/// Folds every guard-shaped statement of one member, or reports that the member keeps a read of
/// the switch field the fold cannot rewrite.
///
/// The walk is whole-member: every expression of every statement (nested bodies included) is
/// scanned for a read of `field`, and a read that is not the outer condition of a guard this
/// function folded makes the whole member unfoldable — a partial fold would leave the remaining
/// reads pointing at a field the caller hides. Guards nested inside any statement structure
/// (`if`, loops, `try`, `switch` arms) fold where they stand.
pub(crate) fn fold_member_guards(
    statements: &[Stmt],
    field: &str,
    budget: &mut Budget,
) -> Result<Option<AssertMemberFold>, stop::StopReason> {
    let mut fold = Fold {
        field,
        reads: Vec::new(),
        budget,
    };
    let Some(statements) = fold.stmts(statements)? else {
        return Ok(None);
    };
    Ok(Some(AssertMemberFold {
        statements,
        reads: fold.reads,
    }))
}

/// One member's fold: the field being rewritten away, the reads consumed so far, and the budget
/// every walked node is charged to.
struct Fold<'a> {
    field: &'a str,
    reads: Vec<u32>,
    budget: &'a mut Budget,
}

impl Fold<'_> {
    /// One statement list, folded in order.
    fn stmts(&mut self, statements: &[Stmt]) -> Result<Option<Vec<Stmt>>, stop::StopReason> {
        let mut folded = Vec::with_capacity(statements.len());
        for statement in statements {
            stop::charge(
                self.budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(statement.origin.primary().bci()),
            )?;
            let Some(statement) = self.stmt(statement)? else {
                return Ok(None);
            };
            folded.push(statement);
        }
        Ok(Some(folded))
    }

    /// One statement: the guard shape folded, every other statement rebuilt around its folded
    /// nested bodies and its own checked expressions.
    fn stmt(&mut self, statement: &Stmt) -> Result<Option<Stmt>, stop::StopReason> {
        if let Some(assert) = self.guard_assert(statement) {
            return Ok(Some(assert));
        }
        let origin = statement.origin.clone();
        let kind = match &statement.kind {
            StmtKind::Declare {
                ty,
                source_type_name,
                name,
                value,
            } => {
                let Some(value) = self.opt_expr(value.as_ref())? else {
                    return Ok(None);
                };
                StmtKind::Declare {
                    ty: ty.clone(),
                    source_type_name: source_type_name.clone(),
                    name: name.clone(),
                    value,
                }
            }
            StmtKind::Assign { name, value } => {
                let Some(value) = self.expr(value)? else {
                    return Ok(None);
                };
                StmtKind::Assign {
                    name: name.clone(),
                    value,
                }
            }
            StmtKind::Expr(expr) => {
                let Some(expr) = self.expr(expr)? else {
                    return Ok(None);
                };
                StmtKind::Expr(expr)
            }
            StmtKind::FieldAssign {
                receiver,
                name,
                op,
                value,
            } => {
                let Some(receiver) = self.opt_expr(receiver.as_ref())? else {
                    return Ok(None);
                };
                let Some(value) = self.expr(value)? else {
                    return Ok(None);
                };
                StmtKind::FieldAssign {
                    receiver,
                    name: name.clone(),
                    op: *op,
                    value,
                }
            }
            StmtKind::IndexAssign {
                array,
                index,
                op,
                value,
            } => {
                let Some(array) = self.expr(array)? else {
                    return Ok(None);
                };
                let Some(index) = self.expr(index)? else {
                    return Ok(None);
                };
                let Some(value) = self.expr(value)? else {
                    return Ok(None);
                };
                StmtKind::IndexAssign {
                    array,
                    index,
                    op: *op,
                    value,
                }
            }
            StmtKind::ConstructorCall { target, args } => {
                let Some(args) = self.exprs(args)? else {
                    return Ok(None);
                };
                StmtKind::ConstructorCall {
                    target: *target,
                    args,
                }
            }
            StmtKind::Return { value } => {
                let Some(value) = self.opt_expr(value.as_ref())? else {
                    return Ok(None);
                };
                StmtKind::Return { value }
            }
            StmtKind::Break { .. } | StmtKind::Continue { .. } | StmtKind::Fallback { .. } => {
                statement.kind.clone()
            }
            StmtKind::Throw { value } => {
                let Some(value) = self.expr(value)? else {
                    return Ok(None);
                };
                StmtKind::Throw { value }
            }
            StmtKind::Assert { cond, message } => {
                let Some(cond) = self.expr(cond)? else {
                    return Ok(None);
                };
                let Some(message) = self.opt_expr(message.as_ref())? else {
                    return Ok(None);
                };
                StmtKind::Assert { cond, message }
            }
            StmtKind::If {
                cond,
                then_body,
                else_body,
            } => {
                let Some(cond) = self.expr(cond)? else {
                    return Ok(None);
                };
                let Some(then_body) = self.stmts(then_body)? else {
                    return Ok(None);
                };
                let Some(else_body) = self.stmts(else_body)? else {
                    return Ok(None);
                };
                StmtKind::If {
                    cond,
                    then_body,
                    else_body,
                }
            }
            StmtKind::While { label, cond, body } | StmtKind::DoWhile { label, cond, body } => {
                let Some(cond) = self.expr(cond)? else {
                    return Ok(None);
                };
                let Some(body) = self.stmts(body)? else {
                    return Ok(None);
                };
                let while_ = matches!(statement.kind, StmtKind::While { .. });
                if while_ {
                    StmtKind::While {
                        label: label.clone(),
                        cond,
                        body,
                    }
                } else {
                    StmtKind::DoWhile {
                        label: label.clone(),
                        cond,
                        body,
                    }
                }
            }
            StmtKind::For {
                label,
                init,
                cond,
                update,
                body,
            } => {
                let Some(init) = self.stmt(init)? else {
                    return Ok(None);
                };
                let Some(cond) = self.expr(cond)? else {
                    return Ok(None);
                };
                let Some(update) = self.stmt(update)? else {
                    return Ok(None);
                };
                let Some(body) = self.stmts(body)? else {
                    return Ok(None);
                };
                StmtKind::For {
                    label: label.clone(),
                    init: Box::new(init),
                    cond,
                    update: Box::new(update),
                    body,
                }
            }
            StmtKind::ForEach {
                label,
                ty,
                name,
                iterable,
                body,
            } => {
                let Some(iterable) = self.expr(iterable)? else {
                    return Ok(None);
                };
                let Some(body) = self.stmts(body)? else {
                    return Ok(None);
                };
                StmtKind::ForEach {
                    label: label.clone(),
                    ty: ty.clone(),
                    name: name.clone(),
                    iterable,
                    body,
                }
            }
            StmtKind::Switch { value, arms } => {
                let Some(value) = self.expr(value)? else {
                    return Ok(None);
                };
                let mut folded_arms = Vec::with_capacity(arms.len());
                for arm in arms {
                    let Some(body) = self.stmts(&arm.body)? else {
                        return Ok(None);
                    };
                    folded_arms.push(SwitchArm {
                        keys: arm.keys.clone(),
                        labels: arm.labels.clone(),
                        default: arm.default,
                        fall_through: arm.fall_through,
                        body,
                    });
                }
                StmtKind::Switch {
                    value,
                    arms: folded_arms,
                }
            }
            StmtKind::Try {
                resources,
                catches,
                body,
                finally_body,
            } => {
                let mut folded_resources = Vec::with_capacity(resources.len());
                for resource in resources {
                    let Some(value) = self.expr(&resource.value)? else {
                        return Ok(None);
                    };
                    folded_resources.push(crate::ast::ResourceDecl {
                        ty: resource.ty.clone(),
                        name: resource.name.clone(),
                        value,
                    });
                }
                let Some(body) = self.stmts(body)? else {
                    return Ok(None);
                };
                let mut folded_catches = Vec::with_capacity(catches.len());
                for catch in catches {
                    let Some(catch_body) = self.stmts(&catch.body)? else {
                        return Ok(None);
                    };
                    folded_catches.push(CatchClause {
                        ty: catch.ty.clone(),
                        name: catch.name.clone(),
                        body: catch_body,
                    });
                }
                let Some(finally_body) = self.opt_stmts(finally_body.as_deref())? else {
                    return Ok(None);
                };
                StmtKind::Try {
                    resources: folded_resources,
                    catches: folded_catches,
                    body,
                    finally_body,
                }
            }
            StmtKind::Synchronized { lock, body } => {
                let Some(lock) = self.expr(lock)? else {
                    return Ok(None);
                };
                let Some(body) = self.stmts(body)? else {
                    return Ok(None);
                };
                StmtKind::Synchronized { lock, body }
            }
        };
        Ok(Some(Stmt { kind, origin }))
    }

    /// An optional statement list: `None` input folds to `None` output without refusing.
    fn opt_stmts(
        &mut self,
        statements: Option<&[Stmt]>,
    ) -> Result<Option<Option<Vec<Stmt>>>, stop::StopReason> {
        match statements {
            None => Ok(Some(None)),
            Some(statements) => self.stmts(statements).map(Some),
        }
    }

    /// One expression tree, charged per node, refusing when it reads the switch field outside a
    /// folded guard.
    fn expr(&mut self, expr: &Expr) -> Result<Option<Expr>, stop::StopReason> {
        for node in expr_nodes(expr) {
            stop::charge(
                self.budget,
                CountedBudgetDimension::IrItems,
                1,
                Some(node.origin.primary().bci()),
            )?;
            if matches!(&node.kind, ExprKind::Field { name, .. } if name == self.field) {
                self.reads.push(node.origin.primary().bci());
                return Ok(None);
            }
        }
        Ok(Some(expr.clone()))
    }

    /// [`Fold::expr`] over an optional expression.
    fn opt_expr(&mut self, expr: Option<&Expr>) -> Result<Option<Option<Expr>>, stop::StopReason> {
        match expr {
            None => Ok(Some(None)),
            Some(expr) => self.expr(expr).map(Some),
        }
    }

    /// [`Fold::expr`] over an argument list.
    fn exprs(&mut self, exprs: &[Expr]) -> Result<Option<Vec<Expr>>, stop::StopReason> {
        let mut folded = Vec::with_capacity(exprs.len());
        for expr in exprs {
            let Some(expr) = self.expr(expr)? else {
                return Ok(None);
            };
            folded.push(expr);
        }
        Ok(Some(folded))
    }

    /// Whether one statement is the guard shape, and the `assert` it folds to when it is.
    ///
    /// The shape is exactly javac's lowering: `if (!X.$assertionsDisabled) { if (<failing cond>)
    /// throw new AssertionError(<message>); }` — no `else` on either branch, no statement beside
    /// the inner `if` or the `throw`. Anything else (an extra statement in the guard, a message
    /// the construction could not present as one expression, a nested test) is not this lowering
    /// and the statement keeps the text it has.
    fn guard_assert(&mut self, statement: &Stmt) -> Option<Stmt> {
        let StmtKind::If {
            cond,
            then_body,
            else_body,
        } = &statement.kind
        else {
            return None;
        };
        if !else_body.is_empty() {
            return None;
        }
        let ExprKind::Not { value: switch } = &cond.kind else {
            return None;
        };
        let ExprKind::Field { receiver, name } = &switch.kind else {
            return None;
        };
        if name != self.field {
            return None;
        }
        // The receiver is a type name in source form (`A1`, `p.Outer`); the physical owner of the
        // read is the caller's census to prove, so the spelling itself is not matched here.
        if !matches!(receiver.kind, ExprKind::Path(_)) {
            return None;
        }
        // The read the fold consumes is the guard's own `getstatic`: its BCI is what the caller's
        // census states, so the node's own primary anchor is the one pushed back.
        let read_bci = switch.origin.primary().bci();
        let [inner] = then_body.as_slice() else {
            return None;
        };
        let StmtKind::If {
            cond: failing,
            then_body: throw_body,
            else_body: throw_else,
        } = &inner.kind
        else {
            return None;
        };
        if !throw_else.is_empty() {
            return None;
        }
        let [throwing] = throw_body.as_slice() else {
            return None;
        };
        let StmtKind::Throw { value } = &throwing.kind else {
            return None;
        };
        let ExprKind::New {
            ty,
            qualifier: None,
            member_name: None,
            diamond: false,
            args,
        } = &value.kind
        else {
            return None;
        };
        if ty != ASSERTION_ERROR || args.len() > 1 {
            return None;
        }
        let cond = negate_condition(failing)?;
        let message = args.first().map(assertion_message);
        self.reads.push(read_bci);
        Some(Stmt {
            kind: StmtKind::Assert { cond, message },
            origin: statement.origin.clone(),
        })
    }
}

/// The source condition of an `assert`, negated from the failing condition the guard tested.
///
/// Two shapes invert exactly: a `!`-node is the value it negated, and a comparison is the
/// comparison with the opposite operator — the two forms javac's branch instructions leave the
/// region structure holding. Any other boolean-presented expression is negated by wrapping it in
/// `!`, which states precisely `!(the tested condition)`; an expression this layer cannot state
/// a negation of refuses the fold.
fn negate_condition(failing: &Expr) -> Option<Expr> {
    match &failing.kind {
        ExprKind::Not { value } => Some((**value).clone()),
        ExprKind::Binary { op, left, right } if op.is_comparison() => {
            let op = op.negated().expect("a comparison has its negation");
            Some(Expr {
                kind: ExprKind::Binary {
                    op,
                    left: left.clone(),
                    right: right.clone(),
                },
                origin: failing.origin.clone(),
                presented: failing.presented.clone(),
            })
        }
        _ => wrap_not(failing),
    }
}

/// The `!(…)` spelling of one proved boolean condition.
fn wrap_not(condition: &Expr) -> Option<Expr> {
    if condition.presented != Some(Type::Boolean) {
        return None;
    }
    Some(Expr {
        kind: ExprKind::Not {
            value: Box::new(condition.clone()),
        },
        origin: condition.origin.clone(),
        presented: Some(Type::Boolean),
    })
}

/// The message expression of one folded guard: the construction's single argument, with the
/// `(java.lang.Object)` position cast removed.
///
/// The cast is the call site's own conversion (`AssertionError(Object)` is the descriptor javac
/// selected), not an instruction — no `checkcast` runs for it — and the `assert` message position
/// performs no conversion the constructor would not, so the argument's own expression is the
/// message. Any other cast stays: a downcast is a real instruction with real failure semantics.
fn assertion_message(argument: &Expr) -> Expr {
    match &argument.kind {
        ExprKind::Cast { ty, value } if matches!(ty, Type::Reference(name) if name == "java.lang.Object") => {
            (**value).clone()
        }
        _ => argument.clone(),
    }
}

/// Every expression node of one tree, in pre-order.
fn expr_nodes(expr: &Expr) -> ExprNodes<'_> {
    ExprNodes { stack: vec![expr] }
}

/// The pre-order walk behind [`expr_nodes`].
struct ExprNodes<'a> {
    stack: Vec<&'a Expr>,
}

impl<'a> Iterator for ExprNodes<'a> {
    type Item = &'a Expr;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        let mut push = |expr: &'a Expr| self.stack.push(expr);
        match &node.kind {
            ExprKind::Local(_)
            | ExprKind::Integer(_)
            | ExprKind::IntegerConstantName { .. }
            | ExprKind::Boolean(_)
            | ExprKind::Long(_)
            | ExprKind::Float(_)
            | ExprKind::Double(_)
            | ExprKind::Str(_)
            | ExprKind::Null
            | ExprKind::ClassLiteral { .. }
            | ExprKind::Path(_)
            | ExprKind::QualifiedThis { .. }
            | ExprKind::Super { .. } => {}
            ExprKind::LocalAssign { value, .. } => push(value),
            ExprKind::ArrayAssign { target, value, .. } => {
                push(target);
                push(value);
            }
            ExprKind::InstanceOf { value, .. } => push(value),
            ExprKind::Call { receiver, args, .. } => {
                if let Some(receiver) = receiver {
                    push(receiver);
                }
                for arg in args {
                    push(arg);
                }
            }
            ExprKind::New {
                qualifier, args, ..
            } => {
                if let Some(qualifier) = qualifier {
                    push(qualifier);
                }
                for arg in args {
                    push(arg);
                }
            }
            ExprKind::Lambda { body, .. } => push(body),
            ExprKind::MethodReference { qualifier, .. } => push(qualifier),
            ExprKind::Field { receiver, .. } => push(receiver),
            ExprKind::Index { array, index } => {
                push(array);
                push(index);
            }
            ExprKind::PostfixUpdate { target, .. } => push(target),
            ExprKind::ArrayLength { array } => push(array),
            ExprKind::NewArray {
                lengths,
                initializers,
                ..
            } => {
                for length in lengths {
                    push(length);
                }
                if let Some(initializers) = initializers {
                    for initializer in initializers {
                        push(initializer);
                    }
                }
            }
            ExprKind::Binary { left, right, .. } => {
                push(left);
                push(right);
            }
            ExprKind::Conditional {
                test,
                when_true,
                when_false,
            } => {
                push(test);
                push(when_true);
                push(when_false);
            }
            ExprKind::Concat { parts } => {
                for part in parts {
                    push(&part.value);
                }
            }
            ExprKind::Cast { value, .. } | ExprKind::Not { value } | ExprKind::Neg { value } => {
                push(value)
            }
        }
        Some(node)
    }
}

/// The one proved switch-initialization line of a `<clinit>` program, when the program holds
/// exactly one.
///
/// The line is javac's own lowering, exactly as this layer's build presents it:
/// `$assertionsDisabled = (!X.class.desiredAssertionStatus() ? 1 : 0) % 2 != 0;` — the ternary
/// of the two `iconst` arms, the `Z` store's `% 2 != 0` narrowing, and no receiver on the write.
/// Matching the presented shape keeps the proof on this layer's own verdicts: a build change
/// that presents the write differently stops matching, and the class keeps its verbatim text.
pub(crate) fn switch_line(
    statements: &[Stmt],
    field: &str,
    budget: &mut Budget,
) -> Result<Option<AssertSwitchLine>, stop::StopReason> {
    let mut found: Option<AssertSwitchLine> = None;
    for (position, statement) in statements.iter().enumerate() {
        stop::charge(
            budget,
            CountedBudgetDimension::IrItems,
            1,
            Some(statement.origin.primary().bci()),
        )?;
        let StmtKind::FieldAssign {
            receiver: None,
            name,
            op: AssignOp::Assign,
            value,
        } = &statement.kind
        else {
            continue;
        };
        if name != field {
            continue;
        }
        let Some(class_literal) = switch_line_class(value, budget)? else {
            continue;
        };
        if found.is_some() {
            // Two writes of this shape to one field is not a switch this class initialized once;
            // the field stays physical.
            return Ok(None);
        }
        found = Some(AssertSwitchLine {
            class_literal,
            write_bci: statement.origin.primary().bci(),
            position,
        });
    }
    Ok(found)
}

/// The class literal of the initialization value's `desiredAssertionStatus()` call.
fn switch_line_class(
    value: &Expr,
    budget: &mut Budget,
) -> Result<Option<String>, stop::StopReason> {
    stop::charge(
        budget,
        CountedBudgetDimension::IrItems,
        1,
        Some(value.origin.primary().bci()),
    )?;
    // `… % 2 != 0`
    let ExprKind::Binary {
        op: BinaryOp::NotEqual,
        left,
        right,
    } = &value.kind
    else {
        return Ok(None);
    };
    if !matches!(&right.kind, ExprKind::Integer(0)) {
        return Ok(None);
    }
    // `(!X.class.desiredAssertionStatus() ? 1 : 0) % 2`
    let ExprKind::Binary {
        op: BinaryOp::Remainder,
        left: conditional,
        right: modulus,
    } = &left.kind
    else {
        return Ok(None);
    };
    if !matches!(&modulus.kind, ExprKind::Integer(2)) {
        return Ok(None);
    }
    let ExprKind::Conditional {
        test,
        when_true,
        when_false,
    } = &conditional.kind
    else {
        return Ok(None);
    };
    if !matches!(&when_true.kind, ExprKind::Integer(1))
        || !matches!(&when_false.kind, ExprKind::Integer(0))
    {
        return Ok(None);
    }
    let ExprKind::Not { value: call } = &test.kind else {
        return Ok(None);
    };
    let ExprKind::Call {
        receiver: Some(receiver),
        name,
        args,
    } = &call.kind
    else {
        return Ok(None);
    };
    if name != "desiredAssertionStatus" || !args.is_empty() {
        return Ok(None);
    }
    let ExprKind::ClassLiteral { ty } = &receiver.kind else {
        return Ok(None);
    };
    Ok(Some(ty.clone()))
}
