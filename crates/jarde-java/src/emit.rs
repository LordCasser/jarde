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

use crate::ast::{Expr, ExprKind, Stmt, StmtKind};
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
                self.expr(receiver)?;
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
                    emitter.expr(receiver)?;
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
                emitter.expr(qualifier)?;
                emitter.put("::", at)?;
                emitter.put(name, at)
            }
            ExprKind::Field { receiver, name } => {
                emitter.expr(receiver)?;
                emitter.put(".", at)?;
                emitter.put(name, at)
            }
            ExprKind::Index { array, index } => {
                emitter.expr(array)?;
                emitter.put("[", at)?;
                emitter.expr(index)?;
                emitter.put("]", at)
            }
            ExprKind::Binary { op, left, right } => {
                emitter.expr(left)?;
                emitter.put(&format!(" {} ", op.spell()), at)?;
                emitter.expr(right)
            }
            ExprKind::Not { value } => {
                emitter.put("!", at)?;
                emitter.expr(value)
            }
        })
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
        }
    }
}

/// The indentation of one depth: four spaces, fixed, because this emitter never reflows.
fn indent_text(indent: usize) -> String {
    "    ".repeat(indent)
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
}
