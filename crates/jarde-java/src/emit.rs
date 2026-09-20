//! ④ The emitter: text, segment table, escaping, comments and the output budget, produced by the
//! same writes (P3 1.2 decisions 1–4).
//!
//! # Text and positions cannot disagree
//!
//! Every node is written through [`Emitter::node`], which records the byte range it wrote and the
//! anchors it wrote them for, in the same call that put the bytes in the buffer. There is no second
//! pass over the finished text and no way for the two to drift: a segment exists only because a
//! write happened, and a write to a node always records one. Nested nodes nest their spans, which is
//! why [`crate::source_map::SourceMap::covering`] returns the outermost writer of a byte and
//! [`crate::source_map::SourceMap::of_bci`] returns every writer of an anchor.
//!
//! # The budget is checked at every write entry, before the write
//!
//! [`Emitter::put`] is the only way text enters the buffer: it polls the run, checks the output
//! bound for exactly the bytes it is about to append, and charges them. When either refuses, it
//! **discards the whole buffer** — text and segments — and returns [`StopReason`], so a stopped run
//! cannot hand out a partial artifact that looks like a produced one, and the caller has no success
//! state to mistake it for. That is also why a refusal inside a node is not an error at all but a
//! stop: this emitter stops mid-node exactly like the probe that decided the route did.
//!
//! # Escaping and comments
//!
//! String literals are escaped by UTF-16 code unit, so a supplementary character becomes a surrogate
//! pair (`😀` → `\ud83d\ude00`) and every control character becomes `\uXXXX` rather than a raw byte
//! the artifact would carry literally. A `\` is escaped, which is what keeps a source string that
//! already reads `\u0041` from turning into `A` when the artifact is compiled again.
//!
//! Comments are the other lexical hazard: Java processes `\uXXXX` *before* it lexes, so a comment
//! containing `\u000a` ends the line it is on. The emitter therefore writes comments through
//! [`comment_text`], which drops the one character that can start an escape and every character that
//! ends a line — a reason string from a diagnostic may contain both, and neither is worth an
//! unparsable artifact.
//!
//! # What the envelope is
//!
//! The artifact is the method *body*: a comment naming the method and the evidence that produced it,
//! then the statements in a block. Reconstructing the method's signature needs the descriptor parsed
//! into types, which is a 3.x presentation decision with its own evidence (generics, `throws`,
//! annotations); claiming a signature this layer cannot prove would be exactly the guess the rest of
//! this crate refuses to make.

use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::model::PhysicalMethodId;

use crate::ast::{BinaryOp, Expr, ExprKind, Stmt, StmtKind};
use crate::declaration::Declaration;
use crate::facts::RecoveryFacts;
use crate::source_map::{OriginSet, Segment, SourceMap};
use crate::stop::{StopReason, poll};

/// One produced artifact: the text and the segment table of the same emission.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Emitted {
    pub(crate) text: String,
    pub(crate) source_map: SourceMap,
    /// How many bytes the recovery run wrote, which is what a stopped run reports.
    pub(crate) written: u64,
    /// How many statements of the body this emission wrote as Java. A [`StmtKind::Fallback`] is not
    /// one of them: it writes a reason and the bytecode indexes it quotes. This is the fact
    /// [`crate::report`] classifies a committed artifact's content from, and it travels *with* the
    /// text because both are the emission's: a run that stops mid-emission hands out neither.
    pub(crate) statements: usize,
}

/// Emits one method body.
///
/// `declaration` is what [`crate::declaration`] read of the member's declaration, when the run could
/// read one: it is written into the envelope, because that comment is the only place in this
/// artifact that can state what the member *is* without claiming a signature this layer cannot prove
/// (the descriptor is not parsed into types here — that is 3.x's presentation question).
///
/// `member` is the identity of the body being presented, as the payload's own declaration states it
/// (P3 3.2): every anchor of this body that names no member of its own is recorded as an anchor **of
/// this member**, which is what makes BCI 3 here tellable from BCI 3 inside a callee's body. `None`
/// when the run read no member header, in which case the table states that no member is known rather
/// than one it did not read.
pub(crate) fn emit(
    stmts: &[Stmt],
    facts: &RecoveryFacts,
    declaration: Option<&Declaration>,
    member: Option<&PhysicalMethodId>,
    budget: &mut Budget,
) -> Result<Emitted, StopReason> {
    let mut emitter = Emitter::new(budget, member);
    emitter.envelope(facts, declaration)?;
    emitter.stmts(stmts, 1)?;
    emitter.put("}\n", None)?;
    Ok(emitter.finish())
}

/// The one writer of the artifact.
struct Emitter<'a> {
    budget: &'a mut Budget,
    text: String,
    segments: Vec<Segment>,
    written: u64,
    limit: u64,
    /// The member body every anchor of this emission belongs to, when the payload stated one.
    member: Option<&'a PhysicalMethodId>,
    /// How many statements of the body have been written as Java so far: the fact
    /// [`Emitted::statements`] publishes once every write succeeded.
    statements: usize,
}

impl<'a> Emitter<'a> {
    fn new(budget: &'a mut Budget, member: Option<&'a PhysicalMethodId>) -> Self {
        let limit = budget.limits().output_bytes;
        Self {
            budget,
            text: String::new(),
            segments: Vec::new(),
            written: 0,
            limit,
            member,
            statements: 0,
        }
    }

    /// The comment envelope and the body's opening brace; anchored at no node, so mapped to none.
    fn envelope(
        &mut self,
        facts: &RecoveryFacts,
        declaration: Option<&Declaration>,
    ) -> Result<(), StopReason> {
        let method = facts.method();
        self.put(
            &format!(
                "// @method {}{}\n",
                comment_text(method.name()),
                comment_text(method.descriptor())
            ),
            None,
        )?;
        // What the class file declares the member to be, when the run could read it (P3 2.3): the
        // one fact that tells an interface's `default` method from a class's ordinary one, stated
        // where a reader of the artifact meets it first.
        if let Some(declaration) = declaration {
            let class = declaration
                .declaring_class
                .as_deref()
                .map(|class| comment_text(&class.replace('/', ".")));
            let line = match class {
                Some(class) => format!(
                    "// @declaration {} of `{class}`, member flags {:#06x}\n",
                    declaration.form.spell(),
                    declaration.member_flags
                ),
                None => format!(
                    "// @declaration {}, member flags {:#06x}\n",
                    declaration.form.spell(),
                    declaration.member_flags
                ),
            };
            self.put(&line, None)?;
        }
        self.put(
            "// recovered from bytecode; presentation is not claimed to compile\n",
            None,
        )?;
        self.put("{\n", None)
    }

    /// Writes one node's text, recording its span against the same anchors.
    ///
    /// This is where the body being presented is stated for its own anchors (P3 3.2): the node's
    /// anchors name no member when a rule built them (only a rule that read *another* member's body
    /// has one to name), and the member of the whole artifact is the one the payload's declaration
    /// states. Recording happens in the same call as the write, so a segment cannot exist without
    /// the member it belongs to.
    fn node(
        &mut self,
        origin: &OriginSet,
        write: impl FnOnce(&mut Self) -> Result<(), StopReason>,
    ) -> Result<(), StopReason> {
        let start = self.text.len();
        write(self)?;
        let end = self.text.len();
        if end > start {
            let origin = origin.in_body(self.member);
            self.segments.push(Segment::new(start, end, origin));
        }
        Ok(())
    }

    /// Appends the statements of one body at one indentation depth.
    fn stmts(&mut self, stmts: &[Stmt], indent: usize) -> Result<(), StopReason> {
        for stmt in stmts {
            self.node(&stmt.origin, |emitter| emitter.stmt(stmt, indent))?;
        }
        Ok(())
    }

    /// Appends one statement.
    fn stmt(&mut self, stmt: &Stmt, indent: usize) -> Result<(), StopReason> {
        let at = Some(stmt.origin.primary().bci());
        let pad = indent_text(indent);
        // What a statement is, stated where statements are written: a fallback writes the reason and
        // the bytecode it could not present, so it is not one. Everything else this emitter spells —
        // a declaration, an assignment, a call, a constructor call, `return` and the control-flow
        // statements — is. The count is only ever read out of a finished [`Emitted`], so a stop
        // inside this call discards it with the text.
        if !matches!(stmt.kind, StmtKind::Fallback { .. }) {
            self.statements += 1;
        }
        match &stmt.kind {
            StmtKind::Declare { ty, name, value } => {
                self.put(&pad, at)?;
                self.put(ty.spell(), at)?;
                self.put(" ", at)?;
                self.put(name, at)?;
                if let Some(value) = value {
                    self.put(" = ", at)?;
                    self.expr(value)?;
                }
                self.put(";\n", at)
            }
            StmtKind::Assign { name, value } => {
                self.put(&pad, at)?;
                self.put(name, at)?;
                self.put(" = ", at)?;
                self.expr(value)?;
                self.put(";\n", at)
            }
            StmtKind::Expr(expr) => {
                self.put(&pad, at)?;
                self.expr(expr)?;
                self.put(";\n", at)
            }
            StmtKind::FieldAssign {
                receiver,
                name,
                value,
            } => {
                self.put(&pad, at)?;
                // A write's receiver is a receiver position exactly like a read's.
                self.operand(receiver, PRIMARY)?;
                self.put(".", at)?;
                self.put(name, at)?;
                self.put(" = ", at)?;
                self.expr(value)?;
                self.put(";\n", at)
            }
            StmtKind::Return { value } => {
                self.put(&pad, at)?;
                self.put("return", at)?;
                if let Some(value) = value {
                    self.put(" ", at)?;
                    self.expr(value)?;
                }
                self.put(";\n", at)
            }
            StmtKind::ConstructorCall { target, args } => {
                // `super(…)` or `this(…)`, decided by `init@1` and never by a convention: the
                // receiver is the frames' own uninitialized `this`, and the spellings are two
                // different programs.
                self.put(&pad, at)?;
                self.put(target.spell(), at)?;
                self.put("(", at)?;
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        self.put(", ", at)?;
                    }
                    self.expr(arg)?;
                }
                self.put(");\n", at)
            }
            StmtKind::If {
                cond,
                then_body,
                else_body,
            } => {
                self.put(&pad, at)?;
                self.put("if (", at)?;
                self.expr(cond)?;
                self.put(") {\n", at)?;
                self.stmts(then_body, indent + 1)?;
                self.put(&pad, at)?;
                self.put("}", at)?;
                if else_body.is_empty() {
                    self.put("\n", at)
                } else {
                    self.put(" else {\n", at)?;
                    self.stmts(else_body, indent + 1)?;
                    self.put(&pad, at)?;
                    self.put("}\n", at)
                }
            }
            StmtKind::While { cond, body } => {
                self.put(&pad, at)?;
                self.put("while (", at)?;
                self.expr(cond)?;
                self.put(") {\n", at)?;
                self.stmts(body, indent + 1)?;
                self.put(&pad, at)?;
                self.put("}\n", at)
            }
            StmtKind::DoWhile { cond, body } => {
                self.put(&pad, at)?;
                self.put("do {\n", at)?;
                self.stmts(body, indent + 1)?;
                self.put(&pad, at)?;
                self.put("} while (", at)?;
                self.expr(cond)?;
                self.put(");\n", at)
            }
            StmtKind::Try { resources, body } => {
                self.put(&pad, at)?;
                self.put("try (", at)?;
                for (index, resource) in resources.iter().enumerate() {
                    if index > 0 {
                        self.put("; ", at)?;
                    }
                    // The declaration's own text is anchored where the value it stores is produced:
                    // the header is the only place that initialisation runs, and the segment says
                    // which instruction it came from.
                    self.node(&resource.value.origin, |emitter| {
                        emitter.put(resource.ty.spell(), at)?;
                        emitter.put(" ", at)?;
                        emitter.put(&resource.name, at)?;
                        emitter.put(" = ", at)?;
                        emitter.expr(&resource.value)
                    })?;
                }
                self.put(") {\n", at)?;
                self.stmts(body, indent + 1)?;
                self.put(&pad, at)?;
                self.put("}\n", at)
            }
            StmtKind::Synchronized { lock, body } => {
                self.put(&pad, at)?;
                self.put("synchronized (", at)?;
                self.expr(lock)?;
                self.put(") {\n", at)?;
                self.stmts(body, indent + 1)?;
                self.put(&pad, at)?;
                self.put("}\n", at)
            }
            StmtKind::Switch { value, arms } => {
                self.put(&pad, at)?;
                self.put("switch (", at)?;
                self.expr(value)?;
                self.put(") {\n", at)?;
                for arm in arms {
                    // One label per key, then the no-match label when this arm is the default too.
                    // Labels nest no further: the arm's statements are written one level in.
                    let label_pad = indent_text(indent + 1);
                    for key in &arm.keys {
                        self.put(&label_pad, at)?;
                        self.put(&format!("case {key}:\n"), at)?;
                    }
                    if arm.default {
                        self.put(&label_pad, at)?;
                        self.put("default:\n", at)?;
                    }
                    // The `break` sits with the arm's own statements, one level in from its labels.
                    let body_pad = indent_text(indent + 2);
                    if arm.body.is_empty() {
                        // An empty arm — a case whose target is the switch's own join — has to end
                        // in a `break` of its own, or it would fall into the next arm's code.
                        self.put(&body_pad, at)?;
                        self.put("break;\n", at)?;
                    } else {
                        self.stmts(&arm.body, indent + 2)?;
                        // A `break` after a `return` would be unreachable; every other arm needs
                        // one, because a Java case does fall into the case that follows it.
                        let returns = matches!(
                            arm.body.last().map(|stmt| &stmt.kind),
                            Some(StmtKind::Return { .. })
                        );
                        if !returns {
                            self.put(&body_pad, at)?;
                            self.put("break;\n", at)?;
                        }
                    }
                }
                self.put(&pad, at)?;
                self.put("}\n", at)
            }
            StmtKind::Fallback { reason, bcis } => {
                self.put(&pad, at)?;
                self.put("// @bytecode", at)?;
                for bci in bcis {
                    self.put(&format!(" {bci}"), at)?;
                }
                self.put("\n", at)?;
                self.put(&pad, at)?;
                self.put(&format!("// {}", comment_text(reason)), at)?;
                self.put("\n", at)
            }
        }
    }

    /// Appends one expression.
    fn expr(&mut self, expr: &Expr) -> Result<(), StopReason> {
        let at = Some(expr.origin.primary().bci());
        self.node(&expr.origin, |emitter| match &expr.kind {
            ExprKind::Local(name) => emitter.put(name, at),
            ExprKind::Integer(value) => emitter.put(&value.to_string(), at),
            ExprKind::Boolean(value) => emitter.put(if *value { "true" } else { "false" }, at),
            ExprKind::Long(value) => emitter.put(&format!("{value}L"), at),
            ExprKind::Str(value) => emitter.put(&format!("\"{}\"", escape_string(value)), at),
            ExprKind::Null => emitter.put("null", at),
            ExprKind::Path(path) => emitter.put(path, at),
            ExprKind::Call {
                receiver,
                name,
                args,
            } => {
                if let Some(receiver) = receiver {
                    // The receiver position: the `.` that follows binds tighter than every operator
                    // and than a lambda, so the base keeps its own group or the text regroups it.
                    emitter.operand(receiver, PRIMARY)?;
                    emitter.put(".", at)?;
                }
                emitter.put(name, at)?;
                emitter.put("(", at)?;
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        emitter.put(", ", at)?;
                    }
                    emitter.expr(arg)?;
                }
                emitter.put(")", at)
            }
            ExprKind::New { ty, args } => {
                emitter.put("new ", at)?;
                emitter.put(ty, at)?;
                emitter.put("(", at)?;
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        emitter.put(", ", at)?;
                    }
                    emitter.expr(arg)?;
                }
                emitter.put(")", at)
            }
            ExprKind::Lambda { params, body } => {
                // The parameter list is written with its types: the target type of a recovered
                // lambda is not always declared, and the descriptor that states these types is the
                // same evidence the record reads back.
                emitter.put("(", at)?;
                for (index, param) in params.iter().enumerate() {
                    if index > 0 {
                        emitter.put(", ", at)?;
                    }
                    emitter.put(param.ty.spell(), at)?;
                    emitter.put(" ", at)?;
                    emitter.put(&param.name, at)?;
                }
                emitter.put(") -> ", at)?;
                emitter.expr(body)
            }
            ExprKind::MethodReference { qualifier, name } => {
                // The qualifier position, for the same reason as a call's receiver: `::` is a
                // suffix over a `Primary`, an `ExpressionName` or a type name.
                emitter.operand(qualifier, PRIMARY)?;
                emitter.put("::", at)?;
                emitter.put(name, at)
            }
            ExprKind::Field { receiver, name } => {
                // The field receiver is a receiver position exactly like a call's.
                emitter.operand(receiver, PRIMARY)?;
                emitter.put(".", at)?;
                emitter.put(name, at)
            }
            ExprKind::Index { array, index } => {
                // The indexee position: `[` is a suffix, so the array's own text has to end where
                // the text does. The **index** is delimited by the brackets and needs nothing.
                emitter.operand(array, PRIMARY)?;
                emitter.put("[", at)?;
                emitter.expr(index)?;
                emitter.put("]", at)
            }
            ExprKind::Binary { op, left, right } => {
                emitter.binary_operand(left, *op, Side::Left)?;
                emitter.put(&format!(" {} ", op.spell()), at)?;
                emitter.binary_operand(right, *op, Side::Right)
            }
            ExprKind::Not { value } => {
                // The operand of `!` is at the unary level, so a looser value keeps its own group:
                // `!a + b` would be `(!a) + b`, another tree than `!(a + b)`.
                emitter.put("!", at)?;
                emitter.operand(value, UNARY)
            }
        })
    }

    /// Appends one operand in a position that accepts only text binding at least as tightly as
    /// `least`, in the parentheses that keep its own group when it binds looser.
    ///
    /// This is the printer's grouping rule stated once, and the position is what states the
    /// requirement: [`Self::binary_operand`] is the binary-operator position on this same scale,
    /// while a receiver, a method reference's qualifier and an indexee demand [`PRIMARY`] — the
    /// level at which a following `.`, `::` or `[` applies to the whole expression — and the
    /// operand of `!` demands [`UNARY`]. Every other position this printer has needs nothing: a
    /// call's arguments and an array's index are delimited by `,`/`)` and `[`/`]`, a condition,
    /// switch selector and `synchronized` lock sit inside their own parentheses, and a return
    /// value, an initialiser, an assignment's right-hand side and a lambda body each take the whole
    /// expression to the end of the text around it.
    ///
    /// The parentheses are written around the operand's own node, which is where they belong: the
    /// segment table still records the operand's text against the operand's anchors, exactly as it
    /// records the operator's own spelling, and no node's anchors move.
    fn operand(&mut self, operand: &Expr, least: u8) -> Result<(), StopReason> {
        let at = Some(operand.origin.primary().bci());
        let grouped = expression_binding(&operand.kind) < least;
        if grouped {
            self.put("(", at)?;
        }
        self.expr(operand)?;
        if grouped {
            self.put(")", at)?;
        }
        Ok(())
    }

    /// Appends one operand of a binary expression, in the parentheses Java's own precedence and
    /// associativity require it to keep the tree's grouping.
    ///
    /// The expression tree is the layer's proof; the text is what a caller receives and recompiles,
    /// and Java groups an unparenthesised text by its own rules. Concatenating the operand's text
    /// therefore prints **another program** where the operand is a binary expression of looser (or
    /// equal, on the right) binding: `i * (2 - d * i)` reaches the buffer as `i * 2 - d * i`, which
    /// is `(i * 2) - (d * i)`. Every binary operator this subset writes is left-associative (JLS
    /// 15.17, 15.18, 15.20), so one comparison covers every pair: an operand keeps its own grouping
    /// only if it binds tighter than its parent — and an equal-precedence operand on the *right*
    /// does not, because the text would read it as part of the parent's own group
    /// (`a - (b - c)` is not `(a - b) - c`).
    ///
    /// This is the binary-operator position of [`Self::operand`]'s scale: the operand on the left
    /// needs the parent's own level, and the one on the right needs to bind strictly tighter.
    fn binary_operand(
        &mut self,
        operand: &Expr,
        parent: BinaryOp,
        side: Side,
    ) -> Result<(), StopReason> {
        let parent = binary_binding(parent);
        let least = match side {
            Side::Left => parent,
            Side::Right => parent + 1,
        };
        self.operand(operand, least)
    }

    /// The only way text enters the buffer.
    ///
    /// `at` is the node the write belongs to, or `None` for the envelope, which maps to no node. A
    /// refusal discards the buffer: text and segments both, so nothing partial survives it.
    fn put(&mut self, text: &str, at: Option<u32>) -> Result<(), StopReason> {
        if text.is_empty() {
            return Ok(());
        }
        if let Err(stop) = poll(self.budget, at) {
            self.discard();
            return Err(stop);
        }
        let bytes = u64::try_from(text.len()).unwrap_or(u64::MAX);
        let allowed = self
            .budget
            .check(CountedBudgetDimension::OutputBytes, bytes)
            .is_ok()
            && self
                .budget
                .charge(CountedBudgetDimension::OutputBytes, bytes)
                .is_ok();
        if !allowed {
            let stop = StopReason::Budget {
                dimension: CountedBudgetDimension::OutputBytes,
                written: self.written,
                limit: self.limit,
                at,
            };
            self.discard();
            return Err(stop);
        }
        self.written += bytes;
        self.text.push_str(text);
        Ok(())
    }

    /// Throws away everything this emission wrote, so that no partial artifact can be handed out.
    ///
    /// The bytes were written and then thrown away, so `written` keeps counting them: the report of
    /// a stopped run states how far it got, and only the text and the segments are discarded.
    fn discard(&mut self) {
        self.text.clear();
        self.segments.clear();
    }

    /// The artifact, once every write succeeded.
    fn finish(self) -> Emitted {
        let mut source_map = SourceMap::default();
        for segment in self.segments {
            source_map.record(segment);
        }
        Emitted {
            text: self.text,
            source_map,
            written: self.written,
            statements: self.statements,
        }
    }
}

/// The indentation of one depth: four spaces, fixed, because this emitter never reflows.
fn indent_text(indent: usize) -> String {
    "    ".repeat(indent)
}

/// Which operand of a binary expression is being written.
///
/// The side is read for one fact: an operand of the same binding power as its parent is regrouped
/// by the text only on the right, because Java's binary operators are all left-associative.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Side {
    Left,
    Right,
}

/// The level at which a following `.`, `::` or `[` applies to the whole expression: a Primary or an
/// ExpressionName (JLS 15.8, 6.5.6) is the tightest text this subset writes.
const PRIMARY: u8 = 6;

/// The level of the unary `!` (JLS 15.15.6): tighter than every binary operator, looser than a
/// primary — `!b.f()` reads as `!(b.f())`, so a `!` in a primary position keeps its own group.
const UNARY: u8 = 5;

/// How tightly one whole expression this subset writes binds: a larger value binds tighter, and a
/// position that accepts only tighter text is written through [`Emitter::operand`].
///
/// Only the order of these values matters — the printer compares two of them — and the levels are
/// Java's own (JLS 15). A lambda is an AssignmentExpression (15.27), looser than every operator and
/// every suffix; the binary operators are 15.17–15.20; `!` is 15.15.6; everything else is a Primary
/// or an ExpressionName. The comparison is a total rule rather than a table over pairs because
/// every binary operator this subset writes is left-associative, and the AST's subset carries no
/// assignment, conditional or boolean-and/or node whose associativity would need a second rule.
fn expression_binding(kind: &ExprKind) -> u8 {
    match kind {
        ExprKind::Lambda { .. } => 0,
        ExprKind::Binary { op, .. } => binary_binding(*op),
        ExprKind::Not { .. } => UNARY,
        // A call, `new`, a field read, an array read, a literal, a name, a type name: every one of
        // them is read whole before any suffix or operator applies.
        _ => PRIMARY,
    }
}

/// How tightly Java binds one binary operator, in the units [`expression_binding`] compares: the
/// four values are Java's own groups (JLS 15.17 multiplicative, 15.18 additive, 15.20 relational
/// then equality).
fn binary_binding(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Remainder => 4,
        BinaryOp::Add | BinaryOp::Subtract => 3,
        BinaryOp::Less | BinaryOp::LessOrEqual | BinaryOp::Greater | BinaryOp::GreaterOrEqual => 2,
        BinaryOp::Equal | BinaryOp::NotEqual => 1,
    }
}

/// One string literal's text, escaped by UTF-16 code unit.
///
/// The unit matters: a Java string literal is a sequence of UTF-16 code units, so a supplementary
/// character must leave this function as the two escapes its surrogate pair is, not as the one
/// `\uXXXX` its code point would be (which is not a Java escape at all).
///
/// Published rather than private because it is a *rule* and not an implementation detail: 3.3's
/// controlled recompilation compares what this crate writes against what a compiler reads back, and
/// a checker has to be able to state the escaping it is checking.
pub fn escape_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for unit in value.encode_utf16() {
        match unit {
            0x22 => escaped.push_str("\\\""),
            0x5c => escaped.push_str("\\\\"),
            0x08 => escaped.push_str("\\b"),
            0x09 => escaped.push_str("\\t"),
            0x0a => escaped.push_str("\\n"),
            0x0c => escaped.push_str("\\f"),
            0x0d => escaped.push_str("\\r"),
            0x20..=0x7e => escaped.push(char::from_u32(u32::from(unit)).unwrap_or('?')),
            other => escaped.push_str(&format!("\\u{other:04x}")),
        }
    }
    escaped
}

/// One comment's text: no character that can start a Unicode escape, and no character that ends a
/// line.
///
/// Both are real traps rather than style: Java decodes `\uXXXX` before it lexes, so a comment
/// holding one is not a comment any more, and a raw newline in a reason string would end the
/// comment and leave the rest of the message as code.
pub fn comment_text(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\\' => '/',
            '\n' | '\r' | '\u{2028}' | '\u{2029}' => ' ',
            character if character.is_control() => ' ',
            character => character,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, Expr, ExprKind, Stmt, StmtKind};
    use crate::facts::{MethodFacts, RecoveryFacts};
    use crate::source_map::Origin;
    use jarde_reader::budget::Limits;

    fn budget_with(output_bytes: u64) -> Budget {
        Budget::new(Limits {
            output_bytes,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        })
    }

    fn facts() -> RecoveryFacts {
        RecoveryFacts::new(MethodFacts::new("sample", "()V", 0))
    }

    fn body() -> Vec<Stmt> {
        vec![Stmt::new(
            StmtKind::Expr(Expr::direct(
                ExprKind::Call {
                    receiver: None,
                    name: "run".to_string(),
                    args: Vec::new(),
                },
                4,
            )),
            OriginSet::new(Origin::direct(4)),
        )]
    }

    #[test]
    fn a_string_literal_escapes_by_utf16_unit() {
        assert_eq!(escape_string("a\"b"), "a\\\"b");
        assert_eq!(escape_string("a\\b"), "a\\\\b");
        assert_eq!(escape_string("a\nb"), "a\\nb");
        assert_eq!(escape_string("\u{0}\u{7}\u{7f}"), "\\u0000\\u0007\\u007f");
        assert_eq!(escape_string("\u{2028}"), "\\u2028");
        assert_eq!(escape_string("😀"), "\\ud83d\\ude00", "a surrogate pair");
        assert_eq!(
            escape_string("\\u0041"),
            "\\\\u0041",
            "an escape in the source stays an escape"
        );
        assert!(
            !escape_string("\u{1}").contains('\u{1}'),
            "no raw control byte survives"
        );
    }

    #[test]
    fn a_comment_cannot_carry_an_escape_or_end_its_own_line() {
        assert_eq!(comment_text("a\\u000ab"), "a/u000ab");
        assert_eq!(comment_text("a\nb"), "a b");
        assert_eq!(comment_text("a\u{0}b"), "a b");
    }

    #[test]
    fn a_refused_write_empties_the_buffer_it_had_already_filled() {
        // The report of a stop carries no text because [`crate::report`] builds one only from a
        // successful emission — this is the second half of the same property, stated where it can be
        // observed: the buffer itself holds nothing after a refusal, so no consumer that ever gets
        // hold of an emitter can read a half-written node out of it.
        let mut budget = budget_with(32);
        let mut emitter = Emitter::new(&mut budget, None);
        emitter
            .put("// a first line\n", None)
            .expect("within the bound");
        assert_eq!(emitter.text, "// a first line\n");
        let stop = emitter
            .put("// a second line that will not fit\n", None)
            .expect_err("eleven bytes over the bound");
        assert!(matches!(stop, StopReason::Budget { .. }), "{stop:?}");
        assert!(
            emitter.text.is_empty(),
            "the buffer is discarded, not handed out half written: {:?}",
            emitter.text
        );
        assert!(emitter.segments.is_empty(), "and so is the segment table");
        assert!(emitter.written > 0, "the stop still states how far it got");
    }

    #[test]
    fn the_budget_is_checked_before_every_write_and_a_refusal_discards_the_buffer() {
        // Exactly the bytes this body writes: the envelope, the call and the closing brace.
        let stmts = body();
        let exact = {
            let mut budget = budget_with(1 << 20);
            emit(&stmts, &facts(), None, None, &mut budget)
                .expect("an ample budget writes")
                .written
        };
        let mut budget = budget_with(exact);
        let emitted =
            emit(&stmts, &facts(), None, None, &mut budget).expect("the exact bound is allowed");
        assert_eq!(emitted.written, exact);
        assert!(
            emitted.text.contains("run();"),
            "a run at the bound produced its artifact: {}",
            emitted.text
        );

        let mut budget = budget_with(exact - 1);
        let stop = emit(&stmts, &facts(), None, None, &mut budget).expect_err("one byte short");
        match stop {
            StopReason::Budget {
                dimension,
                written,
                limit,
                ..
            } => {
                assert_eq!(dimension, CountedBudgetDimension::OutputBytes);
                assert_eq!(limit, exact - 1);
                assert!(written < exact, "the stop never claims more than the bound");
            }
            other => panic!("expected the output bound, got {other:?}"),
        }
    }

    #[test]
    fn a_stop_inside_a_node_leaves_nothing_behind() {
        let stmts = vec![Stmt::new(
            StmtKind::Assign {
                name: "local1".to_string(),
                value: Expr::direct(
                    ExprKind::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::direct(ExprKind::Integer(1), 1)),
                        right: Box::new(Expr::direct(ExprKind::Integer(2), 2)),
                    },
                    3,
                ),
            },
            OriginSet::new(Origin::direct(3)),
        )];
        let whole = {
            let mut budget = budget_with(1 << 20);
            emit(&stmts, &facts(), None, None, &mut budget)
                .expect("ample")
                .written
        };
        // Every bound below the artifact's own size refuses somewhere, and at least one of them
        // refuses while the assignment's *expression* is being written: the emitter stops inside a
        // node rather than at a statement boundary, which is where a bound can hide a half-written
        // node if the writer is not the thing that checks.
        let mut inside_a_node = 0usize;
        for bound in 1..whole {
            let mut budget = budget_with(bound);
            match emit(&stmts, &facts(), None, None, &mut budget) {
                Ok(emitted) => panic!("a {bound}-byte bound produced {} bytes", emitted.written),
                Err(StopReason::Budget { written, at, .. }) => {
                    assert!(
                        written <= bound,
                        "the stop never claims more than the bound"
                    );
                    if at == Some(3) {
                        inside_a_node += 1;
                    }
                }
                Err(other) => panic!("expected the output bound, got {other:?}"),
            }
        }
        assert!(
            inside_a_node > 0,
            "some bound stops the emission inside the expression, not between statements"
        );
    }

    #[test]
    fn the_segment_table_covers_the_nodes_in_writing_order() {
        let stmts = body();
        let mut budget = budget_with(1 << 20);
        let emitted = emit(&stmts, &facts(), None, None, &mut budget).expect("ample");
        // Two nodes, because a statement contains its expression, and the table is in completion
        // order: the expression's span is recorded when its own writes finish, the statement's when
        // the indentation and the terminator around it are written too.
        assert_eq!(
            emitted.source_map.len(),
            2,
            "{:#?}",
            emitted.source_map.segments()
        );
        assert_eq!(
            emitted.source_map.segments()[0].text(&emitted.text),
            "run()"
        );
        assert_eq!(
            emitted.source_map.segments()[1].text(&emitted.text),
            "    run();\n"
        );
        assert_eq!(
            emitted.source_map.text_of_bci(&emitted.text, 4),
            vec!["run()", "    run();\n"],
            "and the BCI reaches both, most specific first"
        );
        assert!(emitted.text.starts_with("// @method sample()V\n"));
        assert!(emitted.text.ends_with("}\n"));
    }

    /// Grouping writes two parentheses and moves no anchor: the operand's own segment is its own
    /// text against its own anchors, and the parentheses are bytes of the **parent's** span, exactly
    /// as the operator between the operands is.
    ///
    /// The expression is the shape of the defect (`local1 * (2 - arg0 * local1)`), built here rather
    /// than recovered so that the assertion is about the printer and not about a run: the inner
    /// subtraction's segment must not contain the parentheses that put it in its own group, while
    /// the outer product's span covers them.
    #[test]
    fn grouping_writes_parentheses_into_the_parents_span_and_moves_no_anchor() {
        let inner = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Subtract,
                left: Box::new(Expr::direct(ExprKind::Integer(2), 3)),
                right: Box::new(Expr::new(
                    ExprKind::Binary {
                        op: BinaryOp::Multiply,
                        left: Box::new(Expr::direct(ExprKind::Local("arg0".to_string()), 4)),
                        right: Box::new(Expr::direct(ExprKind::Local("local1".to_string()), 5)),
                    },
                    OriginSet::new(Origin::direct(6)).plus_derived(Origin::derived(5)),
                )),
            },
            OriginSet::new(Origin::direct(7)).plus_derived(Origin::derived(3)),
        );
        let stmts = vec![Stmt::new(
            StmtKind::Assign {
                name: "local1".to_string(),
                value: Expr::new(
                    ExprKind::Binary {
                        op: BinaryOp::Multiply,
                        left: Box::new(Expr::direct(ExprKind::Local("local1".to_string()), 2)),
                        right: Box::new(inner),
                    },
                    OriginSet::new(Origin::direct(8)).plus_derived(Origin::derived(7)),
                ),
            },
            OriginSet::new(Origin::direct(8)),
        )];
        let mut budget = budget_with(1 << 20);
        let emitted = emit(&stmts, &facts(), None, None, &mut budget).expect("ample");
        assert!(
            emitted
                .text
                .contains("local1 = local1 * (2 - arg0 * local1);"),
            "the right operand of the product is a subtraction, so it keeps its own group:\n{}",
            emitted.text
        );
        // One segment per node, in completion order: the five nodes inside the subtraction, the
        // product, the assignment and its expression's outer product — the parentheses add none.
        assert_eq!(
            emitted.source_map.len(),
            8,
            "{:#?}",
            emitted.source_map.segments()
        );
        assert_eq!(
            emitted.source_map.text_of_bci(&emitted.text, 7),
            vec!["2 - arg0 * local1", "local1 * (2 - arg0 * local1)"],
            "the operands' own texts, the inner one unparenthesised: the parentheses belong to the \
             product that needed them"
        );
        assert_eq!(
            emitted.source_map.text_of_bci(&emitted.text, 3),
            vec!["2", "2 - arg0 * local1"],
            "the constant is its own node and the subtraction that read it presents that anchor as \
             derived, exactly as it did before the grouping"
        );
        assert_eq!(
            emitted.source_map.text_of_bci(&emitted.text, 6),
            vec!["arg0 * local1"],
            "the inner product answers for the bytecode that produced it, with no parentheses in \
             its segment"
        );
    }

    // ---------------------------------------------------------------------------------------
    // The positions a subexpression can be written in.
    //
    // Grouping is a property of the **position**, not only of a binary parent: `(a + b).substring(1)`
    // reached the buffer as `a + b.substring(1)` — `a + (b.substring(1))`, another program — because
    // the call branch printed its receiver and then the `.`. These tests build the shapes directly,
    // so every position is exercised even where this subset's own rules cannot reach it today (the
    // `Not` child is always a boolean parameter's load, the indexee is always the enum dispatch
    // table's field read, and a field's receiver is never a concatenation in javac's output).
    // ---------------------------------------------------------------------------------------

    fn local_at(name: &str, bci: u32) -> Expr {
        Expr::direct(ExprKind::Local(name.to_string()), bci)
    }

    fn integer_at(value: i64, bci: u32) -> Expr {
        Expr::direct(ExprKind::Integer(value), bci)
    }

    fn sum_at(left: Expr, right: Expr, bci: u32) -> Expr {
        Expr::direct(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(left),
                right: Box::new(right),
            },
            bci,
        )
    }

    fn call_at(receiver: Option<Expr>, name: &str, args: Vec<Expr>, bci: u32) -> Expr {
        Expr::direct(
            ExprKind::Call {
                receiver: receiver.map(Box::new),
                name: name.to_string(),
                args,
            },
            bci,
        )
    }

    /// One expression as the body's only statement, so a printer shape is asserted without a run.
    fn emitted_value(value: Expr) -> Emitted {
        let stmts = vec![Stmt::new(
            StmtKind::Expr(value),
            OriginSet::new(Origin::direct(1)),
        )];
        let mut budget = budget_with(1 << 20);
        emit(&stmts, &facts(), None, None, &mut budget).expect("an ample budget writes")
    }

    /// One statement, with its own anchors, as the body's whole text.
    fn emitted_stmt(kind: StmtKind, anchors: &[u32]) -> Emitted {
        let mut origin = OriginSet::new(Origin::direct(anchors[0]));
        for bci in &anchors[1..] {
            origin = origin.plus_derived(Origin::derived(*bci));
        }
        let stmts = vec![Stmt::new(kind, origin)];
        let mut budget = budget_with(1 << 20);
        emit(&stmts, &facts(), None, None, &mut budget).expect("an ample budget writes")
    }

    /// Every position whose text is followed by something that binds tighter than a binary
    /// expression — `.`, `::`, `[` — takes the operand's whole text, so a subexpression that binds
    /// looser than a primary keeps its own group in parentheses.
    #[test]
    fn a_suffix_position_keeps_the_group_of_its_operand() {
        let receiver = sum_at(local_at("arg0", 2), local_at("arg1", 3), 4);
        let emitted = emitted_value(Expr::new(
            ExprKind::Call {
                receiver: Some(Box::new(receiver)),
                name: "substring".to_string(),
                args: vec![integer_at(1, 7)],
            },
            OriginSet::new(Origin::direct(8)).plus_derived(Origin::derived(4)),
        ));
        assert!(
            emitted.text.contains("(arg0 + arg1).substring(1);"),
            "the receiver of a call is a receiver position: `arg0 + arg1.substring(1)` is \
             `arg0 + (arg1.substring(1))`, another program:\n{}",
            emitted.text
        );
        assert_eq!(
            emitted.source_map.text_of_bci(&emitted.text, 4),
            vec!["arg0 + arg1", "(arg0 + arg1).substring(1)"],
            "the receiver's own segment is its own text; the parentheses belong to the call"
        );

        let field = emitted_value(Expr::new(
            ExprKind::Field {
                receiver: Box::new(sum_at(local_at("arg0", 2), local_at("arg1", 3), 4)),
                name: "f".to_string(),
            },
            OriginSet::new(Origin::direct(5)),
        ));
        assert!(
            field.text.contains("(arg0 + arg1).f;"),
            "a field read's receiver is a receiver position:\n{}",
            field.text
        );

        let index = emitted_value(Expr::new(
            ExprKind::Index {
                array: Box::new(sum_at(local_at("arg0", 2), local_at("arg1", 3), 4)),
                index: Box::new(integer_at(0, 7)),
            },
            OriginSet::new(Origin::direct(5)),
        ));
        assert!(
            index.text.contains("(arg0 + arg1)[0];"),
            "the indexee is a suffix position and the index is delimited:\n{}",
            index.text
        );

        let reference = emitted_value(Expr::new(
            ExprKind::MethodReference {
                qualifier: Box::new(sum_at(local_at("arg0", 2), local_at("arg1", 3), 4)),
                name: "length".to_string(),
            },
            OriginSet::new(Origin::direct(5)),
        ));
        assert!(
            reference.text.contains("(arg0 + arg1)::length;"),
            "`::` applies to the whole qualifier, so a binary one keeps its group:\n{}",
            reference.text
        );
    }

    /// The operator positions of the same scale: `!` binds tighter than every binary operator, so a
    /// binary operand keeps its group; and a `!` in a receiver position does too, because `!a.f()`
    /// reads as `!(a.f())`.
    #[test]
    fn a_prefix_position_and_a_not_receiver_keep_their_groups() {
        let not = emitted_value(Expr::direct(
            ExprKind::Not {
                value: Box::new(sum_at(local_at("arg0", 2), local_at("arg1", 3), 4)),
            },
            5,
        ));
        assert!(
            not.text.contains("!(arg0 + arg1);"),
            "`!` is a unary operator and its operand is not:\n{}",
            not.text
        );

        let not_receiver = emitted_value(call_at(
            Some(Expr::direct(
                ExprKind::Not {
                    value: Box::new(local_at("arg0", 2)),
                },
                3,
            )),
            "f",
            vec![],
            4,
        ));
        assert!(
            not_receiver.text.contains("(!arg0).f();"),
            "`!a.f()` is `!(a.f())`, so a negation in a receiver position keeps its own group:\n{}",
            not_receiver.text
        );
    }

    /// A lambda is an AssignmentExpression: nothing in this subset binds looser, so one written
    /// where a suffix follows keeps its own group. (Its target type is a separate question this
    /// layer does not answer; the position only owes the tree's grouping.)
    #[test]
    fn a_lambda_in_a_receiver_position_keeps_its_own_group() {
        let lambda = Expr::direct(
            ExprKind::Lambda {
                params: vec![crate::ast::LambdaParam {
                    ty: crate::ast::Type::Reference("java.lang.String".to_string()),
                    name: "p0".to_string(),
                }],
                body: Box::new(call_at(Some(local_at("p0", 3)), "trim", vec![], 3)),
            },
            2,
        );
        let emitted = emitted_value(call_at(
            Some(lambda),
            "applyAsInt",
            vec![local_at("arg0", 5)],
            6,
        ));
        assert!(
            emitted
                .text
                .contains("((java.lang.String p0) -> p0.trim()).applyAsInt(arg0);"),
            "the `->` body ends where the lambda ends, so `applyAsInt` may not be written inside \
             it:\n{}",
            emitted.text
        );
    }

    /// The positions that delimit their operand — a call's argument, an array's index, a condition
    /// (and a switch selector and lock), a return value, an initialiser and a lambda body — gain no
    /// parentheses, and neither does a binary operand whose own level already states the tree.
    #[test]
    fn a_delimited_or_already_grouped_position_gains_no_parentheses() {
        let emitted = emitted_value(call_at(
            None,
            "f",
            vec![sum_at(local_at("arg0", 2), local_at("arg1", 3), 4)],
            5,
        ));
        assert!(
            emitted.text.contains("f(arg0 + arg1);"),
            "an argument is delimited by `,` and `)`, so it needs nothing:\n{}",
            emitted.text
        );

        let index = emitted_value(Expr::new(
            ExprKind::Index {
                array: Box::new(local_at("arg0", 2)),
                index: Box::new(sum_at(local_at("arg1", 3), local_at("arg2", 4), 5)),
            },
            OriginSet::new(Origin::direct(6)),
        ));
        assert!(
            index.text.contains("arg0[arg1 + arg2];"),
            "an index is delimited by `[` and `]`:\n{}",
            index.text
        );

        let returned = emitted_stmt(
            StmtKind::Return {
                value: Some(sum_at(local_at("arg0", 2), local_at("arg1", 3), 4)),
            },
            &[4],
        );
        assert!(
            returned.text.contains("return arg0 + arg1;"),
            "a return value is the whole expression to the `;`:\n{}",
            returned.text
        );

        let declared = emitted_stmt(
            StmtKind::Declare {
                ty: crate::ast::Type::Int,
                name: "local0".to_string(),
                value: Some(sum_at(local_at("arg0", 2), local_at("arg1", 3), 4)),
            },
            &[4],
        );
        assert!(
            declared.text.contains("int local0 = arg0 + arg1;"),
            "an initialiser is the whole expression to the `;`:\n{}",
            declared.text
        );

        let conditional = emitted_stmt(
            StmtKind::If {
                cond: sum_at(local_at("arg0", 2), local_at("arg1", 3), 4),
                then_body: vec![Stmt::new(
                    StmtKind::Expr(call_at(None, "f", vec![], 6)),
                    OriginSet::new(Origin::direct(6)),
                )],
                else_body: vec![Stmt::new(
                    StmtKind::Expr(call_at(None, "g", vec![], 7)),
                    OriginSet::new(Origin::direct(7)),
                )],
            },
            &[4],
        );
        assert!(
            conditional.text.contains("if (arg0 + arg1) {"),
            "a condition is written inside its own parentheses:\n{}",
            conditional.text
        );

        let switch = emitted_stmt(
            StmtKind::Switch {
                value: sum_at(local_at("arg0", 2), local_at("arg1", 3), 4),
                arms: vec![crate::ast::SwitchArm {
                    keys: vec![0],
                    default: false,
                    body: vec![Stmt::new(
                        StmtKind::Expr(call_at(None, "f", vec![], 6)),
                        OriginSet::new(Origin::direct(6)),
                    )],
                }],
            },
            &[4],
        );
        assert!(
            switch.text.contains("switch (arg0 + arg1) {"),
            "a switch selector is written inside its own parentheses:\n{}",
            switch.text
        );

        let lock = emitted_stmt(
            StmtKind::Synchronized {
                lock: sum_at(local_at("arg0", 2), local_at("arg1", 3), 4),
                body: vec![Stmt::new(
                    StmtKind::Expr(call_at(None, "f", vec![], 6)),
                    OriginSet::new(Origin::direct(6)),
                )],
            },
            &[4],
        );
        assert!(
            lock.text.contains("synchronized (arg0 + arg1) {"),
            "a lock is written inside its own parentheses:\n{}",
            lock.text
        );

        let lambda_body = emitted_value(Expr::direct(
            ExprKind::Lambda {
                params: vec![],
                body: Box::new(sum_at(local_at("arg0", 2), local_at("arg1", 3), 4)),
            },
            5,
        ));
        assert!(
            lambda_body.text.contains("() -> arg0 + arg1;"),
            "a lambda body is the whole expression after `->`:\n{}",
            lambda_body.text
        );
    }

    /// The controls that must stay byte-for-byte what they were: a primary receiver and a nested
    /// call gain nothing, an operand that binds tighter than its parent gains nothing, an
    /// equal-precedence operand on the left gains nothing, and a binary whose operand is `!` gains
    /// nothing (`!a == b` is `(!a) == b`).
    #[test]
    fn a_primary_operand_and_an_already_stated_group_gain_nothing() {
        let plain = emitted_value(call_at(Some(local_at("arg0", 2)), "foo", vec![], 3));
        assert!(
            plain.text.contains("arg0.foo();"),
            "a name receiver is a primary:\n{}",
            plain.text
        );

        let chained = emitted_value(call_at(
            Some(call_at(Some(local_at("arg0", 2)), "trim", vec![], 3)),
            "length",
            vec![],
            4,
        ));
        assert!(
            chained.text.contains("arg0.trim().length();"),
            "a call is a primary, so it needs no parentheses:\n{}",
            chained.text
        );

        let tighter = emitted_value(sum_at(
            local_at("arg0", 2),
            Expr::direct(
                ExprKind::Binary {
                    op: BinaryOp::Multiply,
                    left: Box::new(local_at("arg1", 3)),
                    right: Box::new(local_at("arg2", 4)),
                },
                5,
            ),
            6,
        ));
        assert!(
            tighter.text.contains("arg0 + arg1 * arg2;"),
            "a tighter operand already keeps its own group:\n{}",
            tighter.text
        );

        let left = emitted_value(sum_at(
            sum_at(local_at("arg0", 2), local_at("arg1", 3), 4),
            local_at("arg2", 5),
            6,
        ));
        assert!(
            left.text.contains("arg0 + arg1 + arg2;"),
            "an equal-precedence operand on the left is already where left associativity puts \
             it:\n{}",
            left.text
        );

        let not_operand = emitted_value(Expr::direct(
            ExprKind::Binary {
                op: BinaryOp::Equal,
                left: Box::new(Expr::direct(
                    ExprKind::Not {
                        value: Box::new(local_at("arg0", 2)),
                    },
                    3,
                )),
                right: Box::new(local_at("arg1", 4)),
            },
            5,
        ));
        assert!(
            not_operand.text.contains("!arg0 == arg1;"),
            "`!` binds tighter than `==`, so the text already reads as the tree:\n{}",
            not_operand.text
        );
    }
}
