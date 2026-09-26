//! ④ The emitter: text, segment table, escaping, comments and the output budget, produced by the
//! same formatter (P3 1.2 decisions 1–4).
//!
//! # Two passes of one formatter, and why the table is the second one
//!
//! Every node is written through [`Emitter::node`] and every byte through [`Emitter::put`]. The
//! **committing** pass appends the artifact's text, counts the spans it anchored and stores no
//! segment table at all: a run that delivers the necessary text pays nothing for a table it did not
//! select. The **replay** ([`emit_source_map`]) walks the same AST through the same methods, writes
//! no text, verifies every write against the artifact at the offset it states for it, and records
//! the spans the selection holds. So a segment exists only because a write happened in the pass that
//! wrote the artifact, and an offset the table states is an offset into bytes the artifact really
//! holds — the gate is the comparison in [`Emitter::put`], made in the replay at every write.
//!
//! Nested nodes nest their spans, which is why [`crate::source_map::SourceMap::covering`] returns
//! the outermost writer of a byte and [`crate::source_map::SourceMap::of_bci`] returns every writer
//! of an anchor.
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
//! A replay charges no output: it writes none. Its own work is charged to the evidence phase, one
//! `IrItems` per anchored span, and a phase that refuses there ends the replay with the spans it
//! recorded ([`crate::evidence::Materialized`]) — never with a byte of the artifact removed.
//!
//! # Escaping and comments
//!
//! String literals are escaped by Unicode scalar: a proved scalar is written as the character it is
//! (`😀` stays `😀`, and `正在` stays `正在`), and only the characters a literal cannot hold as
//! themselves stay escapes — the delimiter, the backslash, the line terminators and the controls,
//! spelled by [`escape_unit`]'s table (`\n`, `\u0007`) rather than carried as raw bytes. A `\` is
//! escaped, which is what keeps a source string that already reads `\u0041` from turning into `A`
//! when the artifact is compiled again.
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

use crate::ast::{BinaryOp, Expr, ExprKind, Stmt, StmtKind, Type};
use crate::declaration::Declaration;
use crate::evidence::{EvidencePhase, Materialized, SegmentPublication};
use crate::facts::RecoveryFacts;
use crate::source_map::{OriginSet, Segment, SourceMap};
use crate::stop::{StopReason, poll};

/// One produced artifact: the text, and what the emission that wrote it did.
///
/// # The artifact holds no segment table (change `add-demand-driven-core-results`, D3)
///
/// The committing pass never owns a `Vec<Segment>`: the spans it anchored are **counted**, and the
/// table of the ones the request selected is materialized afterwards from the same formatter and the
/// same AST ([`emit_source_map`]). That is what makes the map an optional evidence category like the
/// others — paid for out of the same remaining allowance, stoppable inside, and never the price of
/// delivering the text.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Emitted {
    pub(crate) text: String,
    /// How many spans this emission anchored, whether or not the request selected the table that
    /// holds them: the figure is the *emission's* own work, and the report states it in one summary
    /// whatever the selection was (change `add-demand-driven-core-results`, D1). A count is not a
    /// segment: nothing here is an owning record a caller could read back.
    pub(crate) segments: u64,
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
    let mut emitter = Emitter::commit(budget, member);
    match emitter
        .envelope(facts, declaration)
        .and_then(|()| emitter.body(stmts, declaration))
    {
        Ok(()) => Ok(emitter.finish()),
        // A committing pass has no phase to stop for and no artifact to disagree with: what it
        // states is the run's own refusal.
        Err(Halt::Stop(stop)) => Err(stop),
        Err(Halt::PhaseStopped) => unreachable!("the committing pass runs no evidence phase"),
        Err(Halt::Gate(_)) => unreachable!("the committing pass verifies against no artifact"),
    }
}

/// Emits one already-proved initializer fragment for the class-source adapter.
///
/// The fragment includes its assignment separator so every byte the adapter appends to a field
/// declaration is written and charged by the same expression formatter as a method body. The
/// caller stages every fragment before mutating any declaration, making a stop atomic at group
/// scope.
pub(crate) fn emit_class_initializer_value(
    value: &Expr,
    member: &PhysicalMethodId,
    budget: &mut Budget,
) -> Result<String, StopReason> {
    let mut emitter = Emitter::commit(budget, Some(member));
    let at = Some(value.origin.primary().bci());
    match emitter.put(" = ", at).and_then(|()| emitter.expr(value)) {
        Ok(()) => Ok(emitter.finish().text),
        Err(Halt::Stop(stop)) => Err(stop),
        Err(Halt::PhaseStopped) => unreachable!("initializer emission has no evidence phase"),
        Err(Halt::Gate(_)) => unreachable!("initializer emission does not replay an artifact"),
    }
}

/// Emits already-proved enum-constructor user statements at constructor-body indentation.
///
/// The caller supplies only the same-run AST statements retained by the enum group proof. This
/// keeps their expression precedence and escaping on the ordinary Java emitter path while omitting
/// the method envelope, which the class-source assembler writes from the proved source Signature.
pub(crate) fn emit_class_enum_constructor_statements(
    statements: &[Stmt],
    member: &PhysicalMethodId,
    budget: &mut Budget,
) -> Result<String, StopReason> {
    let mut emitter = Emitter::commit(budget, Some(member));
    match emitter.stmts(statements, 2) {
        Ok(()) => Ok(emitter.finish().text),
        Err(Halt::Stop(stop)) => Err(stop),
        Err(Halt::PhaseStopped) => unreachable!("constructor statement emission has no phase"),
        Err(Halt::Gate(_)) => unreachable!("constructor statement emission has no replay"),
    }
}

/// The evidence phase's own pass over the decided AST: the *same formatter*, writing no text.
///
/// The source map is the one product that cannot be written while the text is: a run that delivered
/// the text and stopped before this pass must keep the text and say so, and a run that did not
/// select the map must own no table at all. So the committed artifact is replayed through the same
/// emitter — the same AST, the same writes, the same order — into a sink that writes **nothing**:
/// it counts the bytes it did not copy, verifies every one of them against the artifact at its own
/// offset, and records the spans the selection holds.
///
/// # Cost, and why a replay is not a second recovery
///
/// No reader, no IR and no rule runs here: the AST is the one [`crate::build`] decided, and the
/// formatter is the one that wrote the artifact. The replay's own work is charged to the phase, one
/// `IrItems` per anchored span, so it competes with the other categories for the same allowance and
/// stops with the prefix it recorded. Nothing is charged to `OutputBytes`: the text is not written
/// a second time, and a replay that stops leaves every byte of the artifact as it was.
///
/// # The consistency gate
///
/// A span is only ever recorded against bytes the artifact really holds: every write is compared,
/// byte for byte, with the artifact at the offset the replay states for it, and the whole replay has
/// to cover exactly the artifact's own length. A disagreement is stated as a stop — never as a map
/// of offsets into text that other writes produced — so a map that cannot be attached to the
/// artifact is not handed out.
// The eight parameters are the eight distinct inputs of one seam — what is being replayed, the
// facts and declaration it is replayed under, the member it belongs to, what kind of publication
// this replay is, the artifact it must agree with, the phase that charges the work and the budget
// that funds it — and bundling any of them into a struct would only move the same values one level
// down for a caller that holds them all anyway.
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_source_map(
    stmts: &[Stmt],
    facts: &RecoveryFacts,
    declaration: Option<&Declaration>,
    member: Option<&PhysicalMethodId>,
    publication: SegmentPublication,
    artifact: &Emitted,
    phase: &mut EvidencePhase,
    budget: &mut Budget,
) -> Result<(SourceMap, Materialized), StopReason> {
    let mut emitter = Emitter::replay(budget, member, &artifact.text, publication, phase);
    let halt = emitter
        .envelope(facts, declaration)
        .and_then(|()| emitter.body(stmts, declaration))
        .err();
    let (map, covered) = emitter.finish_replay();
    match halt {
        // The replay covered the whole AST and the whole artifact: the map holds everything the
        // selection selects.
        None if covered => Ok((map, Materialized::Complete)),
        // The replay ran to the end of the AST but its stream is not the artifact's length: the gate
        // fails, and no map is handed out.
        None => Err(gate_stop(artifact, &map)),
        // The phase refused one more charge: the spans recorded so far are the prefix it delivers,
        // and the artifact is untouched.
        Some(Halt::PhaseStopped) => {
            let reached = phase_prefix(&map);
            Ok((map, reached))
        }
        Some(Halt::Gate(stop)) | Some(Halt::Stop(stop)) => Err(stop),
    }
}

/// The stop one replay states when its stream did not cover the artifact it was verifying against.
///
/// The gate's own code: the two passes of one formatter disagree about what the artifact holds, so
/// the map cannot be handed out and the run states the stop in the vocabulary it states every other
/// stop in. The text the committing pass delivered is not touched by it.
fn gate_stop(artifact: &Emitted, map: &SourceMap) -> StopReason {
    debug_assert!(
        false,
        "the source-map replay did not cover the {} byte artifact ({} span(s) recorded)",
        artifact.written,
        map.len(),
    );
    StopReason::Interrupted {
        code: crate::stop::SOURCE_MAP_MISMATCH_CODE,
        at: None,
    }
}

/// The state a source map the phase stopped inside reports: the prefix of spans it recorded, or
/// nothing at all when it did not reach one whole node of the artifact.
fn phase_prefix(map: &SourceMap) -> Materialized {
    if map.is_empty() {
        Materialized::None
    } else {
        Materialized::Partial {
            delivered: u64::try_from(map.len()).unwrap_or(u64::MAX),
        }
    }
}

/// Why one write of one pass stopped.
///
/// The committing pass has exactly one reason to stop — the run refuses — and the replay has three:
/// the phase that pays for it refuses, the run refuses, or the write disagrees with the artifact the
/// committing pass produced. One enum because every write goes through [`Emitter::put`].
#[derive(Debug)]
enum Halt {
    /// The run refuses: the budget, a cancellation. What a committing pass states as its stop.
    Stop(StopReason),
    /// The evidence phase refused one more charge. The artifact is untouched and the replay hands
    /// back the prefix of spans it recorded.
    PhaseStopped,
    /// A replayed write is not what the committed artifact holds at that offset, or the replay did
    /// not cover the artifact's own length.
    Gate(StopReason),
}

impl From<StopReason> for Halt {
    fn from(stop: StopReason) -> Self {
        Self::Stop(stop)
    }
}

/// The one writer of the artifact, in either of its two modes.
///
/// The committing mode appends text through [`Emitter::put`], checking the output bound before every
/// write and discarding the buffer on a refusal; the replay mode writes nothing, verifies every write
/// against the artifact at its own offset, and records the spans the selection holds — through the
/// same node, statement and expression methods, so the two passes cannot spell text differently.
struct Emitter<'a> {
    budget: &'a mut Budget,
    /// The artifact's text. Empty in a replay: nothing is written a second time.
    text: String,
    /// What a replay writes through, when this emitter is one.
    replay: Option<Replay<'a>>,
    /// How many spans this emission anchored, selected or not.
    anchored: u64,
    written: u64,
    limit: u64,
    /// The member body every anchor of this emission belongs to, when the payload stated one.
    member: Option<&'a PhysicalMethodId>,
    /// How many statements of the body have been written as Java so far: the fact
    /// [`Emitted::statements`] publishes once every write succeeded.
    statements: usize,
}

/// What a source-map replay holds: the artifact it verifies against, the spans it records, and the
/// phase that pays for the work of replaying.
struct Replay<'a> {
    /// The committed artifact: every replayed write is compared with the bytes at its own offset.
    artifact: &'a str,
    /// Which spans of the artifact the selection records.
    publication: SegmentPublication,
    /// The spans recorded so far, in the artifact's own completion order.
    segments: Vec<Segment>,
    /// The evidence phase this replay is charged to, one `IrItems` per anchored span.
    phase: &'a mut EvidencePhase,
}

impl<'a> Emitter<'a> {
    /// The committing pass: text, checked at every write entry.
    fn commit(budget: &'a mut Budget, member: Option<&'a PhysicalMethodId>) -> Self {
        let limit = budget.limits().output_bytes;
        Self {
            budget,
            text: String::new(),
            replay: None,
            anchored: 0,
            written: 0,
            limit,
            member,
            statements: 0,
        }
    }

    /// The replay: no text, and the spans the selection holds.
    fn replay(
        budget: &'a mut Budget,
        member: Option<&'a PhysicalMethodId>,
        artifact: &'a str,
        publication: SegmentPublication,
        phase: &'a mut EvidencePhase,
    ) -> Self {
        let limit = budget.limits().output_bytes;
        Self {
            budget,
            text: String::new(),
            replay: Some(Replay {
                artifact,
                publication,
                segments: Vec::new(),
                phase,
            }),
            anchored: 0,
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
    ) -> Result<(), Halt> {
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
    ///
    /// In a replay the same call *records* the span instead of writing it: the offsets are the
    /// offsets the committing pass wrote ([`Emitter::written`] counts exactly the bytes that pass
    /// appended), the anchors are the same anchors of the same AST node, and the span is recorded
    /// only when the selection holds one of its positions — the driver range meets the source-map
    /// gate in [`SegmentPublication::records`] and nowhere else.
    fn node(
        &mut self,
        origin: &OriginSet,
        write: impl FnOnce(&mut Self) -> Result<(), Halt>,
    ) -> Result<(), Halt> {
        let start = self.written;
        write(self)?;
        let end = self.written;
        if end > start {
            let origin = origin.in_body(self.member);
            self.anchored += 1;
            let Some(replay) = self.replay.as_mut() else {
                // The committing pass records no span: the table is the replay's product.
                return Ok(());
            };
            // The replay's own work is charged to the phase that pays for this category: one
            // `IrItems` per anchored span, charged *before* the span is recorded, so a refusal ends
            // the replay with the prefix it already recorded and never with a record no charge paid
            // for.
            if !replay.phase.may_continue(self.budget) {
                return Err(Halt::PhaseStopped);
            }
            if replay.publication.records(origin.bcis()) {
                crate::demand_counts::record_built(
                    crate::evidence::RecoveryEvidenceKind::SourceMap,
                );
                replay.segments.push(Segment::new(
                    usize::try_from(start).unwrap_or(usize::MAX),
                    usize::try_from(end).unwrap_or(usize::MAX),
                    origin,
                ));
            }
        }
        Ok(())
    }

    /// Appends the statements of one body at one indentation depth.
    fn stmts(&mut self, stmts: &[Stmt], indent: usize) -> Result<(), Halt> {
        for stmt in stmts {
            self.node(&stmt.origin, |emitter| emitter.stmt(stmt, indent))?;
        }
        Ok(())
    }

    /// Writes one body and its closing brace. A class initializer's final void return is the
    /// bytecode terminator, but Java spells that normal completion with the block's closing brace.
    /// The return's origin is retained on that brace so commit and replay publish the same source
    /// span. Only the top-level final statement is eligible; nested and non-final returns remain
    /// ordinary statements.
    fn body(&mut self, stmts: &[Stmt], declaration: Option<&Declaration>) -> Result<(), Halt> {
        let projected_return = declaration
            .is_some_and(|declaration| {
                declaration.form == crate::declaration::DeclarationForm::StaticInitializer
            })
            .then(|| stmts.last())
            .flatten()
            .filter(|stmt| matches!(&stmt.kind, StmtKind::Return { value: None }));

        let body_stmts = projected_return
            .map(|_| &stmts[..stmts.len() - 1])
            .unwrap_or(stmts);
        self.stmts(body_stmts, 1)?;
        if let Some(stmt) = projected_return {
            let at = Some(stmt.origin.primary().bci());
            self.node(&stmt.origin, |emitter| emitter.put("}\n", at))
        } else {
            self.put("}\n", None)
        }
    }

    /// Writes a proved `for` header assignment without the statement terminator.
    fn for_clause(&mut self, stmt: &Stmt) -> Result<(), Halt> {
        let at = Some(stmt.origin.primary().bci());
        match &stmt.kind {
            StmtKind::Declare {
                ty,
                name,
                value: Some(value),
            } => {
                self.put(ty.spell(), at)?;
                self.put(" ", at)?;
                self.put(name, at)?;
                self.put(" = ", at)?;
                self.expr(value)
            }
            StmtKind::Assign { name, value } => {
                self.put(name, at)?;
                self.put(" = ", at)?;
                self.expr(value)
            }
            _ => unreachable!("a for header contains only proved local initialisation/update"),
        }
    }

    /// Appends one statement.
    fn stmt(&mut self, stmt: &Stmt, indent: usize) -> Result<(), Halt> {
        let at = Some(stmt.origin.primary().bci());
        let pad = indent_text(indent);
        // What a statement is, stated where statements are written: a fallback writes the reason and
        // the bytecode it could not present, so it is not one. Everything else this emitter spells —
        // a declaration, an assignment, a call, a constructor call, `return`, `throw` and the control-flow
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
                op,
                value,
            } => {
                self.put(&pad, at)?;
                if let Some(receiver) = receiver {
                    // A qualified write's receiver is a receiver position exactly like a read's.
                    self.operand(receiver, PRIMARY)?;
                    self.put(".", at)?;
                }
                self.put(name, at)?;
                self.put(" ", at)?;
                self.put(op.spell(), at)?;
                self.put(" ", at)?;
                self.expr(value)?;
                self.put(";\n", at)
            }
            // `array[index] = value;` — the left-hand side is the subscript the read is, written in
            // the indexee position for the same reason (`[` is a suffix).
            StmtKind::IndexAssign {
                array,
                index,
                op,
                value,
            } => {
                self.put(&pad, at)?;
                self.operand(array, PRIMARY)?;
                self.put("[", at)?;
                self.expr(index)?;
                self.put("] ", at)?;
                self.put(op.spell(), at)?;
                self.put(" ", at)?;
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
            StmtKind::Break { label } => {
                self.put(&pad, at)?;
                self.put("break", at)?;
                if let Some(label) = label {
                    self.put(" ", at)?;
                    self.put(label, at)?;
                }
                self.put(";\n", at)
            }
            StmtKind::Continue { label } => {
                self.put(&pad, at)?;
                self.put("continue", at)?;
                if let Some(label) = label {
                    self.put(" ", at)?;
                    self.put(label, at)?;
                }
                self.put(";\n", at)
            }
            StmtKind::Throw { value } => {
                self.put(&pad, at)?;
                self.put("throw ", at)?;
                self.expr(value)?;
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
            StmtKind::While { label, cond, body } => {
                self.put(&pad, at)?;
                if let Some(label) = label {
                    self.put(label, at)?;
                    self.put(": ", at)?;
                }
                self.put("while (", at)?;
                self.expr(cond)?;
                self.put(") {\n", at)?;
                self.stmts(body, indent + 1)?;
                self.put(&pad, at)?;
                self.put("}\n", at)
            }
            StmtKind::For {
                label,
                init,
                cond,
                update,
                body,
            } => {
                self.put(&pad, at)?;
                if let Some(label) = label {
                    self.put(label, at)?;
                    self.put(": ", at)?;
                }
                self.put("for (", at)?;
                self.node(&init.origin, |emitter| emitter.for_clause(init))?;
                self.put("; ", at)?;
                self.expr(cond)?;
                self.put("; ", at)?;
                self.node(&update.origin, |emitter| emitter.for_clause(update))?;
                self.put(") {\n", at)?;
                self.stmts(body, indent + 1)?;
                self.put(&pad, at)?;
                self.put("}\n", at)
            }
            StmtKind::ForEach {
                label,
                ty,
                name,
                iterable,
                body,
            } => {
                self.put(&pad, at)?;
                if let Some(label) = label {
                    self.put(label, at)?;
                    self.put(": ", at)?;
                }
                self.put("for (", at)?;
                self.put(ty.spell(), at)?;
                self.put(" ", at)?;
                self.put(name, at)?;
                self.put(" : ", at)?;
                self.expr(iterable)?;
                self.put(") {\n", at)?;
                self.stmts(body, indent + 1)?;
                self.put(&pad, at)?;
                self.put("}\n", at)
            }
            StmtKind::DoWhile { label, cond, body } => {
                self.put(&pad, at)?;
                if let Some(label) = label {
                    self.put(label, at)?;
                    self.put(": ", at)?;
                }
                self.put("do {\n", at)?;
                self.stmts(body, indent + 1)?;
                self.put(&pad, at)?;
                self.put("} while (", at)?;
                self.expr(cond)?;
                self.put(");\n", at)
            }
            StmtKind::Try {
                resources,
                catches,
                body,
                finally_body,
            } => {
                self.put(&pad, at)?;
                // No resources is no header: the statement is the one the exception table states,
                // and writing `try (` with nothing in it would state a guarded shape the class file
                // does not have.
                if resources.is_empty() {
                    self.put("try {\n", at)?;
                } else {
                    self.put("try (", at)?;
                    for (index, resource) in resources.iter().enumerate() {
                        if index > 0 {
                            self.put("; ", at)?;
                        }
                        // The declaration's own text is anchored where the value it stores is
                        // produced: the header is the only place that initialisation runs, and the
                        // segment says which instruction it came from.
                        self.node(&resource.value.origin, |emitter| {
                            emitter.put(resource.ty.spell(), at)?;
                            emitter.put(" ", at)?;
                            emitter.put(&resource.name, at)?;
                            emitter.put(" = ", at)?;
                            emitter.expr(&resource.value)
                        })?;
                    }
                    self.put(") {\n", at)?;
                }
                self.stmts(body, indent + 1)?;
                self.put(&pad, at)?;
                self.put("}", at)?;
                for clause in catches {
                    // The clause is written `} catch (T n) { … }` after the block it catches for:
                    // the type is the row's own class and the parameter the local the handler stores
                    // into, both stated by the node the builder made from the table.
                    self.put(" catch (", at)?;
                    self.put(&clause.ty, at)?;
                    self.put(" ", at)?;
                    self.put(&clause.name, at)?;
                    self.put(") {\n", at)?;
                    self.stmts(&clause.body, indent + 1)?;
                    self.put(&pad, at)?;
                    self.put("}", at)?;
                }
                if let Some(finally_body) = finally_body {
                    self.put(" finally {\n", at)?;
                    self.stmts(finally_body, indent + 1)?;
                    self.put(&pad, at)?;
                    self.put("}", at)?;
                }
                self.put("\n", at)
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
                // How a key is written is decided by the type the selector's own text presents, and by
                // nothing else: a `char` selector writes its keys as the characters the payload's
                // numbers are, and every other selector — an `int` one whose key happens to be `97`
                // included — keeps the number. The class file states the same `int` key either way
                // (`case 97:` over a `char` is legal and means the same character), so the selector's
                // presented type is the only fact that states which spelling the text has.
                let char_selector = matches!(value.presented, Some(Type::Char));
                for arm in arms {
                    // One label per key, then the no-match label when this arm is the default too.
                    // Labels nest no further: the arm's statements are written one level in.
                    let label_pad = indent_text(indent + 1);
                    match &arm.labels {
                        Some(crate::ast::SwitchLabels::Enum(labels)) => {
                            for label in labels {
                                self.put(&label_pad, at)?;
                                self.put(&format!("case {label}:\n"), at)?;
                            }
                        }
                        Some(crate::ast::SwitchLabels::String(labels)) => {
                            for literal in labels {
                                self.put(&label_pad, at)?;
                                self.put(&format!("case \"{}\":\n", escape_string(literal)), at)?;
                            }
                        }
                        None => {
                            for key in &arm.keys {
                                self.put(&label_pad, at)?;
                                self.put(
                                    &format!("case {}:\n", switch_key(*key, char_selector)),
                                    at,
                                )?;
                            }
                        }
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
                        // A case needs its own break only when control can reach the end of its
                        // body. In particular, a loop break inside this switch is already an
                        // abrupt completion, including when both paths of a final `if` exit.
                        if !arm.fall_through && statements_can_complete(&arm.body) {
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
    fn expr(&mut self, expr: &Expr) -> Result<(), Halt> {
        let at = Some(expr.origin.primary().bci());
        self.node(&expr.origin, |emitter| match &expr.kind {
            ExprKind::Local(name) => emitter.put(name, at),
            ExprKind::Integer(value) => emitter.put(&value.to_string(), at),
            ExprKind::Boolean(value) => emitter.put(if *value { "true" } else { "false" }, at),
            ExprKind::Long(value) => emitter.put(&format!("{value}L"), at),
            ExprKind::Float(bits) => emitter.put(&spell_float(*bits), at),
            ExprKind::Double(bits) => emitter.put(&spell_double(*bits), at),
            ExprKind::Str(value) => emitter.put(&format!("\"{}\"", escape_string(value)), at),
            ExprKind::Null => emitter.put("null", at),
            ExprKind::ClassLiteral { ty } => {
                emitter.put(ty, at)?;
                emitter.put(".class", at)
            }
            ExprKind::Path(path) => emitter.put(path, at),
            ExprKind::Super { qualifier } => {
                if let Some(qualifier) = qualifier {
                    emitter.put(qualifier, at)?;
                    emitter.put(".", at)?;
                }
                emitter.put("super", at)
            }
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
            ExprKind::New {
                ty,
                qualifier,
                member_name,
                diamond,
                args,
            } => {
                if let (Some(qualifier), Some(member_name)) = (qualifier, member_name) {
                    emitter.operand(qualifier, PRIMARY)?;
                    emitter.put(".new ", at)?;
                    emitter.put(member_name, at)?;
                    if *diamond {
                        emitter.put("<>", at)?;
                    }
                } else {
                    emitter.put("new ", at)?;
                    emitter.put(ty, at)?;
                }
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
            ExprKind::PostIncrement { target } => {
                // The writable target is already proved by recovery. Its value is written in the
                // same suffix position as a field receiver or array indexee, then the postfix
                // operator is anchored at the update node's own origin.
                emitter.operand(target, PRIMARY)?;
                emitter.put("++", at)
            }
            // `array.length`: `.` is the same suffix `[` is, so the array is written in the indexee
            // position and the member is spelled by the node (P3 2b). It is deliberately not a
            // field-access node: no member is proved here, and the two must not be confused.
            ExprKind::ArrayLength { array } => {
                emitter.operand(array, PRIMARY)?;
                emitter.put(".length", at)
            }
            // `new T[n]`: a Primary like `new T(args)`, with one bracketed length per dimension the
            // instruction allocated. The element type is the node's own (the fact the instruction
            // stated) and the lengths are delimited by their brackets.
            ExprKind::NewArray {
                element,
                lengths,
                initializers,
                total_dimensions,
            } => {
                emitter.put("new ", at)?;
                emitter.put(element.spell(), at)?;
                if let Some(initializers) = initializers {
                    for _ in 0..*total_dimensions {
                        emitter.put("[]", at)?;
                    }
                    emitter.put("{", at)?;
                    for (index, initializer) in initializers.iter().enumerate() {
                        if index > 0 {
                            emitter.put(", ", at)?;
                        }
                        emitter.expr(initializer)?;
                    }
                    emitter.put("}", at)
                } else {
                    for length in lengths {
                        emitter.put("[", at)?;
                        emitter.expr(length)?;
                        emitter.put("]", at)?;
                    }
                    for _ in lengths.len()..usize::from(*total_dimensions) {
                        emitter.put("[]", at)?;
                    }
                    Ok(())
                }
            }
            ExprKind::Binary { op, left, right } => {
                emitter.binary_operand(left, *op, Side::Left)?;
                emitter.put(&format!(" {} ", op.spell()), at)?;
                emitter.binary_operand(right, *op, Side::Right)
            }
            ExprKind::Conditional {
                test,
                when_true,
                when_false,
            } => {
                // `?:` binds more loosely than every binary expression this printer writes and
                // more tightly than a lambda. Each arm is an AssignmentExpression and therefore
                // can contain a conditional of its own without changing its tree.
                emitter.operand(test, CONDITIONAL + 1)?;
                emitter.put(" ? ", at)?;
                emitter.expr(when_true)?;
                emitter.put(" : ", at)?;
                emitter.expr(when_false)
            }
            ExprKind::Concat { parts } => {
                // The parts are written in the chain's own order, each in the position it holds in
                // the `+` expression: the first one is the left operand of the first `+`, and every
                // later part is the right operand of the `+` that adds it. Java's `+` is
                // left-associative, so a part that binds as loosely as `+` keeps its own group on
                // the right — which is what makes `"" + arg0 + arg1 + "!"` (two parts, two
                // conversions) and `"" + (arg0 + arg1) + "!"` (one part that is itself an addition)
                // two different texts of two different programs.
                let string_context = parts.first().is_some_and(|part| !part.is_a_string());
                if string_context {
                    // The empty string the first `+` starts from when the first part is not already
                    // a `String` — the same `""` javac lowers `"" + a` to. The chain has no
                    // instruction behind it (no `append` read it and no BCI produced it), so it is
                    // written by this node and not as a part of its own: the segment that covers it
                    // is this node's, whose anchors are the chain's own instructions.
                    emitter.put("\"\"", at)?;
                }
                for (index, part) in parts.iter().enumerate() {
                    if index > 0 || string_context {
                        emitter.put(" + ", at)?;
                    }
                    let side = if index == 0 && !string_context {
                        Side::Left
                    } else {
                        Side::Right
                    };
                    emitter.binary_operand(&part.value, BinaryOp::Add, side)?;
                }
                Ok(())
            }
            ExprKind::Cast { ty, value } => {
                // The conversion the build decided, written where the value is. The printer writes
                // nothing else here: the type is the node's own, and the operand position is the
                // cast's own (JLS 15.16 — a cast applies to a unary expression), so a looser operand
                // keeps its group (`(int) (a + b)` and never `(int) a + b`).
                emitter.put("(", at)?;
                emitter.put(ty.spell(), at)?;
                emitter.put(") ", at)?;
                emitter.operand(value, UNARY)
            }
            ExprKind::InstanceOf { value, ty } => {
                emitter.operand(value, binary_binding(BinaryOp::Less))?;
                emitter.put(" instanceof ", at)?;
                emitter.put(ty, at)
            }
            ExprKind::Not { value } => {
                // The operand of `!` is at the unary level, so a looser value keeps its own group:
                // `!a + b` would be `(!a) + b`, another tree than `!(a + b)`.
                emitter.put("!", at)?;
                emitter.operand(value, UNARY)
            }
            ExprKind::Neg { value } => {
                // A nested negation or a negative numeric literal needs a lexical group: without
                // it the two minus tokens merge into `--`, which Java parses as decrement (and
                // rejects for a value operand) rather than as two unary negations. A float or
                // double literal's sign is one of its own bits, so `-0.0f` reaches this printer
                // already signed and needs the same protection.
                emitter.put("-", at)?;
                let lexical_group = matches!(&value.kind, ExprKind::Neg { .. })
                    || matches!(&value.kind, ExprKind::Integer(value) if *value < 0)
                    || matches!(&value.kind, ExprKind::Long(value) if *value < 0)
                    || matches!(&value.kind, ExprKind::Float(bits) if *bits >> 31 == 1)
                    || matches!(&value.kind, ExprKind::Double(bits) if *bits >> 63 == 1);
                if lexical_group {
                    emitter.put("(", Some(value.origin.primary().bci()))?;
                    emitter.expr(value)?;
                    emitter.put(")", Some(value.origin.primary().bci()))
                } else {
                    emitter.operand(value, UNARY)
                }
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
    fn operand(&mut self, operand: &Expr, least: u8) -> Result<(), Halt> {
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
    fn binary_operand(&mut self, operand: &Expr, parent: BinaryOp, side: Side) -> Result<(), Halt> {
        let parent = binary_binding(parent);
        let least = match side {
            Side::Left => parent,
            Side::Right => parent + 1,
        };
        self.operand(operand, least)
    }

    /// The only way text enters the buffer, in either mode.
    ///
    /// `at` is the node the write belongs to, or `None` for the envelope, which maps to no node. A
    /// refusal discards the buffer: text and segments both, so nothing partial survives it.
    ///
    /// A replay writes nothing: it verifies that this write is the write the artifact really holds
    /// at this offset, and counts the bytes it did not copy. The one gate between the two passes —
    /// an offset the replay records is only ever an offset into bytes the artifact holds — is this
    /// comparison, made at every write.
    fn put(&mut self, text: &str, at: Option<u32>) -> Result<(), Halt> {
        if text.is_empty() {
            return Ok(());
        }
        if let Some(replay) = self.replay.as_ref() {
            let artifact = replay.artifact;
            let start = usize::try_from(self.written).unwrap_or(usize::MAX);
            let end = start.saturating_add(text.len());
            if artifact.get(start..end) != Some(text) {
                return Err(Halt::Gate(StopReason::Interrupted {
                    code: crate::stop::SOURCE_MAP_MISMATCH_CODE,
                    at,
                }));
            }
            self.written = u64::try_from(end).unwrap_or(u64::MAX);
            return Ok(());
        }
        if let Err(stop) = poll(self.budget, at) {
            self.discard();
            return Err(Halt::Stop(stop));
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
            return Err(Halt::Stop(stop));
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
    }

    /// The artifact, once every write succeeded.
    fn finish(self) -> Emitted {
        Emitted {
            text: self.text,
            segments: self.anchored,
            written: self.written,
            statements: self.statements,
        }
    }

    /// What the replay recorded, and whether it covered the whole artifact.
    ///
    /// The second half of the gate: the replay's own byte count is the artifact's length, so a
    /// stream that stopped short — even one whose every write matched — is not a map of this
    /// artifact. Nothing here is handed out by the replay alone; [`emit_source_map`] states the
    /// stop.
    fn finish_replay(self) -> (SourceMap, bool) {
        let Emitter {
            written, replay, ..
        } = self;
        let Some(replay) = replay else {
            unreachable!("only a replay finishes through the replay path");
        };
        let covered = u64::try_from(replay.artifact.len())
            .map(|length| length == written)
            .unwrap_or(false);
        let mut map = SourceMap::default();
        for segment in replay.segments {
            map.record(segment);
        }
        (map, covered)
    }
}

/// Whether Java control can reach the statement following this sequence. The direct transfers
/// and both sides of an `if` are enough to decide the switch-arm exits this recovery proves.
fn statements_can_complete(body: &[Stmt]) -> bool {
    body.last().is_none_or(|statement| match &statement.kind {
        StmtKind::Break { .. }
        | StmtKind::Continue { .. }
        | StmtKind::Return { .. }
        | StmtKind::Throw { .. } => false,
        StmtKind::If {
            then_body,
            else_body,
            ..
        } => statements_can_complete(then_body) || statements_can_complete(else_body),
        _ => true,
    })
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
const PRIMARY: u8 = 11;

/// The postfix operators (JLS 15.14) bind above unary operators and primary suffixes.
const POSTFIX: u8 = 12;

/// The level of the unary `!` (JLS 15.15.6): tighter than every binary operator, looser than a
/// primary — `!b.f()` reads as `!(b.f())`, so a `!` in a primary position keeps its own group.
const UNARY: u8 = 10;

/// The conditional operator binds below every binary operator and above assignment/lambda text.
const CONDITIONAL: u8 = 1;

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
        ExprKind::PostIncrement { .. } => POSTFIX,
        ExprKind::Conditional { .. } => CONDITIONAL,
        ExprKind::Binary { op, .. } => binary_binding(*op),
        ExprKind::InstanceOf { .. } => binary_binding(BinaryOp::Less),
        // A concatenation is an additive expression: the parts are the operands of its `+`s, so it
        // binds where `+` binds — which is what makes it keep its own group in a receiver, an
        // argument that binds tighter, and the right-hand position of another `+`.
        ExprKind::Concat { .. } => binary_binding(BinaryOp::Add),
        // A cast and a `!` are the same level: both are UnaryExpressions (JLS 15.15–15.16), tighter
        // than every binary operator and looser than a primary, so `(int) a + b` is `((int) a) + b`
        // and a cast in a receiver position keeps its own group.
        ExprKind::Not { .. } | ExprKind::Neg { .. } | ExprKind::Cast { .. } => UNARY,
        // A call, `new`, an array creation (`new int[n]`, whose own `[` groups belong to it), a
        // field read, an array read, a `length` read over an array, a literal, a name, a type name:
        // every one of them is read whole before any suffix or operator applies.
        _ => PRIMARY,
    }
}

/// How tightly Java binds one binary operator, in the units [`expression_binding`] compares. The
/// values descend in Java precedence order from multiplicative through additive, shift, relational,
/// equality, bitwise `&`, `^` and `|`.
fn binary_binding(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Remainder => 9,
        BinaryOp::Add | BinaryOp::Subtract => 8,
        BinaryOp::LeftShift | BinaryOp::RightShift | BinaryOp::UnsignedRightShift => 7,
        BinaryOp::Less | BinaryOp::LessOrEqual | BinaryOp::Greater | BinaryOp::GreaterOrEqual => 6,
        BinaryOp::Equal | BinaryOp::NotEqual => 5,
        BinaryOp::BitwiseAnd => 4,
        BinaryOp::BitwiseXor => 3,
        BinaryOp::BitwiseOr => 2,
    }
}

/// One `float` literal's text, spelled exactly from the raw bits the class file stated.
///
/// The literal is the hexadecimal floating-point form (JLS 3.10.2), which a compiler reads back
/// bit for bit — the one spelling whose round-trip needs no rounding decision at all. The fields
/// are decomposed from the `u32` by integer arithmetic, never through a host `f32`: the sign is
/// the top bit (written as the unary minus Java folds back onto the constant), the 23 mantissa
/// bits shift left one so the 24 significand bits split into one integer hex digit and six
/// fractional ones, and the exponent keeps its own bias of 127. An exponent field of zero — zero
/// and the subnormals — writes integer part `0` at the minimal normal exponent (`p-126`), which
/// is what the encoding itself says those fields mean; the sign of zero survives as its own minus.
/// The suffix is always there, because a spelling that could be read as a double would select the
/// wrong overload.
///
/// Only finite bits reach this function: infinities and the one admitted NaN are presented as the
/// proved constant divisions of these leaves where the constant is read, and an unpresentable NaN
/// is refused before any text exists to spell.
fn spell_float(bits: u32) -> String {
    let sign = if bits >> 31 == 1 { "-" } else { "" };
    let exponent = (bits >> 23) & 0xff;
    let fraction = (bits & 0x007f_ffff) << 1;
    let (integer, biased) = if exponent == 0 {
        (0, -126)
    } else {
        (1, i32::try_from(exponent).unwrap_or(0) - 127)
    };
    format!("{sign}0x{integer:x}.{fraction:06x}p{biased}f")
}

/// One `double` literal's text, by the same raw-bits rule as [`spell_float`]: the 53 significand
/// bits split into one integer hex digit and thirteen fractional ones (52 mantissa bits need no
/// shift), the exponent keeps its bias of 1023, an exponent field of zero writes the subnormal
/// form at `p-1022`, and the suffix is always written.
fn spell_double(bits: u64) -> String {
    let sign = if bits >> 63 == 1 { "-" } else { "" };
    let exponent = (bits >> 52) & 0x7ff;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    let (integer, biased) = if exponent == 0 {
        (0, -1022)
    } else {
        (1, i32::try_from(exponent).unwrap_or(0) - 1023)
    };
    format!("{sign}0x{integer:x}.{fraction:013x}p{biased}d")
}

/// One string literal's text, escaped by Unicode scalar.
///
/// A proved scalar is written as the character it is: `正在` stays `正在`, and a supplementary
/// character stays the one character it is (`😀`, not the two escapes its surrogate pair would be).
/// Only the characters a literal cannot hold as themselves are escaped, and every one of them is
/// inside the BMP, so it is a code unit [`escape_unit`]'s table can spell.
///
/// Published rather than private because it is a *rule* and not an implementation detail: 3.3's
/// controlled recompilation compares what this crate writes against what a compiler reads back, and
/// a checker has to be able to state the escaping it is checking.
pub fn escape_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for scalar in value.chars() {
        // The table spells the units this literal cannot write as themselves; every other scalar, a
        // supplementary one included, is written as the character it is.
        match u16::try_from(scalar) {
            Ok(unit) if needs_escape(unit) => escape_unit(unit, '"', &mut escaped),
            _ => escaped.push(scalar),
        }
    }
    escaped
}

/// Whether one UTF-16 code unit is escaped in a string literal rather than written as itself: the
/// delimiter, the backslash, the line terminators (LF, CR, U+2028, U+2029) and the controls
/// U+0000–U+001F and U+007F.
///
/// The set is the literal's own rule and it is complete on its own: every member is inside the BMP,
/// so [`escape_unit`] can spell it, and no scalar outside it is escaped at all — a unit the table
/// would spell as `\uXXXX` for no reason but not being ASCII stays the character it is.
fn needs_escape(unit: u16) -> bool {
    matches!(unit, 0x00..=0x1f | 0x22 | 0x5c | 0x7f | 0x2028 | 0x2029)
}

/// One `switch` key's own text in a `case` label, from the key the payload states and the type the
/// selector's text presents.
///
/// The key is the signed number `tableswitch`/`lookupswitch` carries, and the class file cannot say
/// which spelling the source gave that number: `case 97:` over a `char` is legal and selects the
/// same character as `case 'a':`. What it does state is the type the selector's text presents, so
/// that is the type the key is written as — a selector that presents `char` writes the character
/// literal, whose escapes [`escape_unit`] gives with this literal's own quote, and every other
/// selector keeps the decimal number. A `char` selector whose key is outside `0..=0xFFFF` keeps the
/// number too: such a key is not any character's value, so no character literal can state it.
///
/// The decision reads [`crate::ast::Expr::presented`] and never the key's own value — the number 97
/// alone types nothing.
fn switch_key(key: i64, char_selector: bool) -> String {
    match u16::try_from(key) {
        Ok(unit) if char_selector => {
            let mut literal = String::from("'");
            escape_unit(unit, '\'', &mut literal);
            literal.push('\'');
            literal
        }
        _ => key.to_string(),
    }
}

/// One UTF-16 code unit's own text as a **character literal** (`'a'`, `'\n'`, `'\''`).
///
/// This is [`switch_key`]'s `char` spelling of one unit and nothing else, for the other place the
/// class file states a character as a number: a position whose required type is `char` and whose
/// value is an in-range `int` constant writes the character that constant stands for (`return 'A';`
/// for the `bipush 65` of a `char`-returning member, P3 2c.30). The build states the value and the
/// type; the *spelling* is this module's, which is why the table stays here rather than being
/// copied into the builder — one table, so the `case` labels and the literals cannot disagree about
/// what a character a class file cannot hold literally is written as.
pub(crate) fn char_literal(unit: u16) -> String {
    switch_key(i64::from(unit), true)
}

/// One UTF-16 code unit's own text inside a literal, appended to `escaped`.
///
/// `quote` is the literal's own delimiter — `"` for a string literal, `'` for a character one — and
/// it is the only difference between the two: the escaping table is one table, so a reader comparing
/// a string literal with a character literal does not have to check two lists against each other.
/// The delimiter is escaped as a two-character escape and the *other* quote is an ordinary character
/// of its own literal, which is why the quote is a parameter rather than a second table.
fn escape_unit(unit: u16, quote: char, escaped: &mut String) {
    match unit {
        0x22 if quote == '"' => escaped.push_str("\\\""),
        0x27 if quote == '\'' => escaped.push_str("\\'"),
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
    use crate::ast::{AssignOp, BinaryOp, ConcatPart, Expr, ExprKind, Stmt, StmtKind, Type};
    use crate::declaration::{Declaration, DeclarationForm};
    use crate::facts::{MethodFacts, RecoveryFacts};
    use crate::source_map::Origin;
    use jarde_reader::budget::Limits;

    fn budget_with(output_bytes: u64) -> Budget {
        Budget::new(Limits {
            output_bytes,
            // The second pass charges the evidence phase one `IrItems` per anchored span; the cases
            // here are about the emitter, so the allowance is ample for both passes.
            ir_items: 1 << 20,
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

    fn declaration(form: DeclarationForm) -> Declaration {
        Declaration {
            form,
            declaring_class: Some("Test".to_string()),
            interface: Some(false),
            member_flags: 0,
        }
    }

    #[test]
    fn a_static_initializer_tail_return_closes_the_block_and_keeps_its_origin() {
        let stmts = vec![
            Stmt::new(
                StmtKind::Expr(Expr::direct(
                    ExprKind::Call {
                        receiver: None,
                        name: "run".to_string(),
                        args: Vec::new(),
                    },
                    4,
                )),
                OriginSet::new(Origin::direct(4)),
            ),
            Stmt::new(
                StmtKind::Return { value: None },
                OriginSet::new(Origin::direct(9)),
            ),
        ];
        let static_initializer = declaration(DeclarationForm::StaticInitializer);
        let mut budget = budget_with(1 << 20);
        let (emitted, map) = artifact(
            &stmts,
            &facts(),
            Some(&static_initializer),
            None,
            SegmentPublication::Whole,
            &mut budget,
        );
        assert!(emitted.text.contains("run();"), "{}", emitted.text);
        assert!(!emitted.text.contains("return;"), "{}", emitted.text);
        assert_eq!(emitted.statements, 1, "{}", emitted.text);
        assert_eq!(map.text_of_bci(&emitted.text, 9), vec!["}\n"]);
    }

    #[test]
    fn an_empty_static_initializer_has_no_statement_but_maps_its_terminator() {
        let stmts = vec![Stmt::new(
            StmtKind::Return { value: None },
            OriginSet::new(Origin::direct(12)),
        )];
        let static_initializer = declaration(DeclarationForm::StaticInitializer);
        let mut budget = budget_with(1 << 20);
        let (emitted, map) = artifact(
            &stmts,
            &facts(),
            Some(&static_initializer),
            None,
            SegmentPublication::Whole,
            &mut budget,
        );
        assert!(emitted.text.ends_with("{\n}\n"), "{}", emitted.text);
        assert_eq!(emitted.statements, 0, "{}", emitted.text);
        assert_eq!(map.text_of_bci(&emitted.text, 12), vec!["}\n"]);
    }

    #[test]
    fn non_initializer_returns_are_not_projected() {
        let stmts = vec![Stmt::new(
            StmtKind::Return { value: None },
            OriginSet::new(Origin::direct(3)),
        )];
        for form in [
            Some(DeclarationForm::StaticMethod),
            Some(DeclarationForm::Constructor),
            None,
        ] {
            let declared = form.map(declaration);
            let mut budget = budget_with(1 << 20);
            let (emitted, map) = artifact(
                &stmts,
                &facts(),
                declared.as_ref(),
                None,
                SegmentPublication::Whole,
                &mut budget,
            );
            assert!(emitted.text.contains("return;"), "{}", emitted.text);
            assert_eq!(emitted.statements, 1, "{}", emitted.text);
            assert_eq!(map.text_of_bci(&emitted.text, 3), vec!["    return;\n"]);
        }
    }

    #[test]
    fn a_nested_return_is_kept_when_the_static_initializer_tail_is_projected() {
        let stmts = vec![
            Stmt::new(
                StmtKind::If {
                    cond: Expr::direct(ExprKind::Boolean(true), 1),
                    then_body: vec![Stmt::new(
                        StmtKind::Return { value: None },
                        OriginSet::new(Origin::direct(2)),
                    )],
                    else_body: Vec::new(),
                },
                OriginSet::new(Origin::direct(1)),
            ),
            Stmt::new(
                StmtKind::Return { value: None },
                OriginSet::new(Origin::direct(8)),
            ),
        ];
        let static_initializer = declaration(DeclarationForm::StaticInitializer);
        let mut budget = budget_with(1 << 20);
        let (emitted, map) = artifact(
            &stmts,
            &facts(),
            Some(&static_initializer),
            None,
            SegmentPublication::Whole,
            &mut budget,
        );
        assert_eq!(
            emitted.text.matches("return;").count(),
            1,
            "{}",
            emitted.text
        );
        assert_eq!(emitted.statements, 2, "{}", emitted.text);
        assert_eq!(map.text_of_bci(&emitted.text, 2), vec!["        return;\n"]);
        assert_eq!(map.text_of_bci(&emitted.text, 8), vec!["}\n"]);
    }

    #[test]
    fn a_static_initializer_projection_keeps_budget_stop_atomic() {
        let stmts = vec![Stmt::new(
            StmtKind::Return { value: None },
            OriginSet::new(Origin::direct(12)),
        )];
        let static_initializer = declaration(DeclarationForm::StaticInitializer);
        let exact = {
            let mut budget = budget_with(1 << 20);
            emit(
                &stmts,
                &facts(),
                Some(&static_initializer),
                None,
                &mut budget,
            )
            .expect("ample")
            .written
        };
        let mut budget = budget_with(exact - 1);
        let stop = emit(
            &stmts,
            &facts(),
            Some(&static_initializer),
            None,
            &mut budget,
        )
        .expect_err("one byte short");
        assert!(matches!(stop, StopReason::Budget { .. }), "{stop:?}");
    }

    #[test]
    fn a_string_literal_escapes_what_it_cannot_hold_literally_and_nothing_else() {
        assert_eq!(escape_string("a\"b"), "a\\\"b");
        assert_eq!(escape_string("a\\b"), "a\\\\b");
        assert_eq!(escape_string("a\nb"), "a\\nb");
        assert_eq!(escape_string("\u{0}\u{7}\u{7f}"), "\\u0000\\u0007\\u007f");
        assert_eq!(escape_string("\u{2028}"), "\\u2028");
        assert_eq!(
            escape_string("😀"),
            "😀",
            "a supplementary scalar is written as the one character it is, not as its surrogate pair"
        );
        assert_eq!(
            escape_string("正在"),
            "正在",
            "a non-ASCII scalar is written as itself: no `\\uXXXX` is invented for it"
        );
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
    fn proven_switch_labels_use_one_closed_spelling_and_keep_integer_evidence() {
        let (string, _) = emitted_stmt(
            StmtKind::Switch {
                value: local_at("arg0", 1),
                arms: vec![crate::ast::SwitchArm {
                    keys: vec![17],
                    labels: Some(crate::ast::SwitchLabels::String(vec![
                        "a\"\\\nb".to_owned(),
                    ])),
                    default: false,
                    fall_through: false,
                    body: vec![],
                }],
            },
            &[2],
        );
        assert!(
            string.text.contains("case \"a\\\"\\\\\\nb\":"),
            "{}",
            string.text
        );
        assert!(!string.text.contains("case 17:"), "{}", string.text);

        let (enum_case, _) = emitted_stmt(
            StmtKind::Switch {
                value: local_at("arg0", 1),
                arms: vec![crate::ast::SwitchArm {
                    keys: vec![17],
                    labels: Some(crate::ast::SwitchLabels::Enum(vec!["READY".to_owned()])),
                    default: false,
                    fall_through: false,
                    body: vec![],
                }],
            },
            &[2],
        );
        assert!(enum_case.text.contains("case READY:"), "{}", enum_case.text);
        assert!(!enum_case.text.contains("case 17:"), "{}", enum_case.text);
    }

    #[test]
    fn a_char_selector_writes_character_keys_and_every_other_selector_stays_decimal() {
        // A `char` selector writes the key as the character it is, delimited and escaped by its own
        // literal's rules: the delimiter is `'` and not the string's `"`, and every character the
        // string literal escapes is escaped the same way — so no raw control character, no raw
        // delimiter and no raw backslash ever reaches the artifact.
        assert_eq!(switch_key(97, true), "'a'");
        assert_eq!(switch_key(0x27, true), "'\\''");
        assert_eq!(
            switch_key(0x22, true),
            "'\"'",
            "the other quote is ordinary"
        );
        assert_eq!(switch_key(0x5c, true), "'\\\\'");
        assert_eq!(switch_key(0x0a, true), "'\\n'");
        assert_eq!(
            switch_key(0x7f, true),
            "'\\u007f'",
            "DEL is not a printable byte"
        );
        assert_eq!(switch_key(0x0, true), "'\\u0000'");
        assert_eq!(switch_key(0x2028, true), "'\\u2028'", "a line separator");
        assert_eq!(switch_key(0xffff, true), "'\\uffff'");
        for (key, raw) in [
            (0x0au16, '\n'),
            (0x0du16, '\r'),
            (0x7fu16, '\u{7f}'),
            (0x2028u16, '\u{2028}'),
            (0x0u16, '\u{0}'),
        ] {
            let literal = switch_key(i64::from(key), true);
            assert!(
                !literal.contains(raw),
                "no raw character of the key survives in `{literal}`"
            );
        }

        // The key's own value types nothing: only a selector that already presents `char` writes a
        // character, and a key outside one code unit's range is not any character's value, so even a
        // `char` selector keeps it decimal.
        assert_eq!(switch_key(97, false), "97");
        assert_eq!(switch_key(-1, true), "-1");
        assert_eq!(switch_key(0x10000, true), "65536");
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
        let mut emitter = Emitter::commit(&mut budget, None);
        emitter
            .put("// a first line\n", None)
            .expect("within the bound");
        assert_eq!(emitter.text, "// a first line\n");
        let stop = emitter
            .put("// a second line that will not fit\n", None)
            .expect_err("eleven bytes over the bound");
        assert!(
            matches!(stop, Halt::Stop(StopReason::Budget { .. })),
            "{stop:?}"
        );
        assert!(
            emitter.text.is_empty(),
            "the buffer is discarded, not handed out half written: {:?}",
            emitter.text
        );
        assert!(
            emitter.replay.is_none(),
            "and the committing pass owns no segment table at all"
        );
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
    fn a_throw_stops_atomically_at_the_output_bound() {
        let stmts = vec![Stmt::new(
            StmtKind::Throw {
                value: Expr::direct(ExprKind::Null, 1),
            },
            OriginSet::new(Origin::direct(1)),
        )];
        let exact = {
            let mut budget = budget_with(1 << 20);
            emit(&stmts, &facts(), None, None, &mut budget)
                .expect("an ample budget writes the throw")
                .written
        };
        let mut budget = budget_with(exact - 1);
        let stop = emit(&stmts, &facts(), None, None, &mut budget)
            .expect_err("one byte short refuses the throw artifact");
        assert!(
            matches!(
                stop,
                StopReason::Budget {
                    dimension: CountedBudgetDimension::OutputBytes,
                    ..
                }
            ),
            "the throw output bound is the reported stop: {stop:?}"
        );
    }

    #[test]
    fn a_throw_source_map_budget_stops_before_publishing_a_partial_node() {
        let stmts = vec![Stmt::new(
            StmtKind::Throw {
                value: Expr::direct(ExprKind::Null, 2),
            },
            OriginSet::new(Origin::direct(1)),
        )];
        let emitted = {
            let mut budget = budget_with(1 << 20);
            emit(&stmts, &facts(), None, None, &mut budget)
                .expect("an ample budget writes the throw")
        };
        let mut budget = Budget::new(Limits {
            ir_items: 0,
            output_bytes: 1 << 20,
            ..Limits::default()
        });
        let mut phase = EvidencePhase::new();
        let (map, reached) = emit_source_map(
            &stmts,
            &facts(),
            None,
            None,
            SegmentPublication::Whole,
            &emitted,
            &mut phase,
            &mut budget,
        )
        .expect("a source-map phase stop delivers a partial evidence result");
        assert_eq!(reached, Materialized::None);
        assert!(map.is_empty(), "the throw node was not partially published");
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
        let (emitted, map) = artifact(
            &stmts,
            &facts(),
            None,
            None,
            SegmentPublication::Whole,
            &mut budget,
        );
        // Two nodes, because a statement contains its expression, and the table is in completion
        // order: the expression's span is recorded when its own writes finish, the statement's when
        // the indentation and the terminator around it are written too.
        assert_eq!(map.len(), 2, "{:#?}", map.segments());
        assert_eq!(map.segments()[0].text(&emitted.text), "run()");
        assert_eq!(map.segments()[1].text(&emitted.text), "    run();\n");
        assert_eq!(
            map.text_of_bci(&emitted.text, 4),
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
        let (emitted, map) = artifact(
            &stmts,
            &facts(),
            None,
            None,
            SegmentPublication::Whole,
            &mut budget,
        );
        assert!(
            emitted
                .text
                .contains("local1 = local1 * (2 - arg0 * local1);"),
            "the right operand of the product is a subtraction, so it keeps its own group:\n{}",
            emitted.text
        );
        // One segment per node, in completion order: the five nodes inside the subtraction, the
        // product, the assignment and its expression's outer product — the parentheses add none.
        assert_eq!(map.len(), 8, "{:#?}", map.segments());
        assert_eq!(
            map.text_of_bci(&emitted.text, 7),
            vec!["2 - arg0 * local1", "local1 * (2 - arg0 * local1)"],
            "the operands' own texts, the inner one unparenthesised: the parentheses belong to the \
             product that needed them"
        );
        assert_eq!(
            map.text_of_bci(&emitted.text, 3),
            vec!["2", "2 - arg0 * local1"],
            "the constant is its own node and the subtraction that read it presents that anchor as \
             derived, exactly as it did before the grouping"
        );
        assert_eq!(
            map.text_of_bci(&emitted.text, 6),
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

    fn binary_at(op: BinaryOp, left: Expr, right: Expr, bci: u32) -> Expr {
        Expr::direct(
            ExprKind::Binary {
                op,
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

    /// One artifact **and its segment table**, as the two passes of the emitter produce them: the
    /// committing pass writes the text, and the replay of the same AST records the spans every node
    /// anchored. The cases below read both, because the table is no longer a product of the write
    /// that produced the text.
    fn artifact(
        stmts: &[Stmt],
        facts: &RecoveryFacts,
        declaration: Option<&Declaration>,
        member: Option<&PhysicalMethodId>,
        publication: SegmentPublication,
        budget: &mut Budget,
    ) -> (Emitted, SourceMap) {
        let emitted =
            emit(stmts, facts, declaration, member, budget).expect("an ample budget writes");
        let mut phase = EvidencePhase::new();
        let (map, reached) = emit_source_map(
            stmts,
            facts,
            declaration,
            member,
            publication,
            &emitted,
            &mut phase,
            budget,
        )
        .expect("the replay of the same AST agrees with the artifact it replayed");
        assert_eq!(
            reached,
            Materialized::Complete,
            "an ample budget materializes the whole table"
        );
        (emitted, map)
    }

    /// The same over the two fixture shapes the cases below build: one expression as the body's only
    /// statement, and one statement with its own anchors.
    fn emitted_value(value: Expr) -> (Emitted, SourceMap) {
        let stmts = vec![Stmt::new(
            StmtKind::Expr(value),
            OriginSet::new(Origin::direct(1)),
        )];
        let mut budget = budget_with(1 << 20);
        artifact(
            &stmts,
            &facts(),
            None,
            None,
            SegmentPublication::Whole,
            &mut budget,
        )
    }

    /// One statement, with its own anchors, as the body's whole text.
    fn emitted_stmt(kind: StmtKind, anchors: &[u32]) -> (Emitted, SourceMap) {
        let mut origin = OriginSet::new(Origin::direct(anchors[0]));
        for bci in &anchors[1..] {
            origin = origin.plus_derived(Origin::derived(*bci));
        }
        let stmts = vec![Stmt::new(kind, origin)];
        let mut budget = budget_with(1 << 20);
        artifact(
            &stmts,
            &facts(),
            None,
            None,
            SegmentPublication::Whole,
            &mut budget,
        )
    }

    #[test]
    fn enhanced_for_spells_its_label_element_and_array_with_folded_origins() {
        let array = Expr::direct(ExprKind::Local("captured".to_owned()), 4)
            .presenting(Type::Reference("int[]".to_owned()));
        let (emitted, map) = emitted_stmt(
            StmtKind::ForEach {
                label: Some("outer".to_owned()),
                ty: Type::Int,
                name: "element".to_owned(),
                iterable: array,
                body: vec![Stmt::new(
                    StmtKind::Continue {
                        label: Some("outer".to_owned()),
                    },
                    OriginSet::new(Origin::direct(20)),
                )],
            },
            &[13, 5, 8, 19, 27],
        );
        assert!(
            emitted
                .text
                .contains("outer: for (int element : captured) {\n        continue outer;\n    }"),
            "{}",
            emitted.text
        );
        for bci in [4, 5, 8, 13, 19, 20, 27] {
            assert!(!map.text_of_bci(&emitted.text, bci).is_empty(), "BCI {bci}");
        }
    }

    #[test]
    fn compound_lvalues_emit_receiver_and_index_once_with_all_origins() {
        let receiver = call_at(None, "select", vec![], 2);
        let rhs = call_at(None, "rhs", vec![], 7);
        let (field, field_map) = emitted_stmt(
            StmtKind::FieldAssign {
                receiver: Some(receiver),
                name: "value".to_string(),
                op: AssignOp::Add,
                value: rhs,
            },
            &[12, 3, 6, 9],
        );
        assert_eq!(field.text.matches("select()").count(), 1, "{}", field.text);
        assert_eq!(field.text.matches("rhs()").count(), 1, "{}", field.text);
        assert!(
            field.text.contains("select().value += rhs();"),
            "{}",
            field.text
        );
        for bci in [2, 3, 6, 7, 9, 12] {
            assert!(
                !field_map.text_of_bci(&field.text, bci).is_empty(),
                "the field update lost BCI {bci}: {:#?}",
                field_map.segments()
            );
        }

        let array = local_at("data", 20);
        let index = call_at(None, "index", vec![], 21);
        let rhs = call_at(None, "rhs", vec![], 24);
        let (element, element_map) = emitted_stmt(
            StmtKind::IndexAssign {
                array,
                index,
                op: AssignOp::Add,
                value: rhs,
            },
            &[29, 22, 23, 25, 27],
        );
        assert_eq!(
            element.text.matches("index()").count(),
            1,
            "{}",
            element.text
        );
        assert_eq!(element.text.matches("rhs()").count(), 1, "{}", element.text);
        assert!(
            element.text.contains("data[index()] += rhs();"),
            "{}",
            element.text
        );
        for bci in [20, 21, 22, 23, 24, 25, 27, 29] {
            assert!(
                !element_map.text_of_bci(&element.text, bci).is_empty(),
                "the array update lost BCI {bci}: {:#?}",
                element_map.segments()
            );
        }
    }

    /// Every position whose text is followed by something that binds tighter than a binary
    /// expression — `.`, `::`, `[` — takes the operand's whole text, so a subexpression that binds
    /// looser than a primary keeps its own group in parentheses.
    #[test]
    fn a_suffix_position_keeps_the_group_of_its_operand() {
        let receiver = sum_at(local_at("arg0", 2), local_at("arg1", 3), 4);
        let (emitted, map) = emitted_value(Expr::new(
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
            map.text_of_bci(&emitted.text, 4),
            vec!["arg0 + arg1", "(arg0 + arg1).substring(1)"],
            "the receiver's own segment is its own text; the parentheses belong to the call"
        );

        let (field, _map) = emitted_value(Expr::new(
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

        let (index, _map) = emitted_value(Expr::new(
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

        let (reference, _map) = emitted_value(Expr::new(
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

        let member = Expr::new(
            ExprKind::New {
                ty: "sample.SimpleOuter$Inner".to_string(),
                qualifier: Some(Box::new(sum_at(
                    local_at("arg0", 12),
                    local_at("arg1", 13),
                    14,
                ))),
                member_name: Some("Inner".to_string()),
                diamond: false,
                args: vec![integer_at(1, 17)],
            },
            OriginSet::new(Origin::direct(18)),
        );
        assert_eq!(
            member.presented,
            Some(Type::Reference("sample.SimpleOuter$Inner".to_string())),
            "the source simple member name must not replace the full semantic allocation type"
        );
        let (constructed, construction_map) = emitted_value(member);
        assert!(
            constructed.text.contains("(arg0 + arg1).new Inner(1);"),
            "qualified creation is a suffix position and keeps the receiver's group:\n{}",
            constructed.text
        );
        assert_eq!(
            construction_map.text_of_bci(&constructed.text, 14),
            vec!["arg0 + arg1"],
            "the qualifier keeps its own span, while grouping remains attached to that receiver"
        );
        assert_eq!(
            construction_map.text_of_bci(&constructed.text, 18),
            vec!["(arg0 + arg1).new Inner(1)"],
            "the construction root maps the complete qualified source expression"
        );

        let generic_member = Expr::new(
            ExprKind::New {
                ty: "sample.SimpleOuter$Inner".to_string(),
                qualifier: Some(Box::new(local_at("arg0", 21))),
                member_name: Some("Inner".to_string()),
                diamond: true,
                args: vec![integer_at(2, 22)],
            },
            OriginSet::new(Origin::direct(23)),
        );
        assert_eq!(
            generic_member.presented,
            Some(Type::Reference("sample.SimpleOuter$Inner".to_string()))
        );
        let (generic, generic_map) = emitted_value(generic_member);
        assert!(
            generic.text.contains("arg0.new Inner<>(2);"),
            "{}",
            generic.text
        );
        assert_eq!(
            generic_map.text_of_bci(&generic.text, 23),
            vec!["arg0.new Inner<>(2)"],
        );
    }

    #[test]
    fn postfix_increment_keeps_nested_writable_targets_and_their_origins() {
        let simple_target = Expr::new(
            ExprKind::Field {
                receiver: Box::new(local_at("arg0", 3)),
                name: "count".to_owned(),
            },
            OriginSet::new(Origin::direct(4)),
        )
        .presenting(Type::Int);
        let negated_increment = Expr::direct(
            ExprKind::Neg {
                value: Box::new(Expr::new(
                    ExprKind::PostIncrement {
                        target: Box::new(simple_target),
                    },
                    OriginSet::new(Origin::direct(5)),
                )),
            },
            6,
        );
        let (negated, _) = emitted_value(negated_increment);
        assert!(
            negated.text.contains("-arg0.count++;"),
            "postfix increment binds more tightly than unary negation:\n{}",
            negated.text
        );

        let receiver = sum_at(
            call_at(None, "left", vec![], 10),
            call_at(None, "right", vec![], 11),
            12,
        );
        let field_target = Expr::new(
            ExprKind::Field {
                receiver: Box::new(receiver.clone()),
                name: "value".to_owned(),
            },
            OriginSet::new(Origin::direct(16)),
        )
        .presenting(Type::Int);
        let field_increment = Expr::new(
            ExprKind::PostIncrement {
                target: Box::new(field_target),
            },
            OriginSet::new(Origin::direct(20)).plus_derived(Origin::derived(17)),
        );
        assert_eq!(field_increment.presented, Some(Type::Int));
        let (field, field_map) = emitted_value(field_increment);
        assert!(
            field.text.contains("(left() + right()).value++;"),
            "the postfix node must preserve a grouped nested receiver:\n{}",
            field.text
        );
        assert!(
            field_map
                .text_of_bci(&field.text, 16)
                .contains(&"(left() + right()).value")
        );
        assert!(
            field_map
                .text_of_bci(&field.text, 20)
                .contains(&"(left() + right()).value++")
        );
        assert!(!field_map.text_of_bci(&field.text, 10).is_empty());
        assert!(!field_map.text_of_bci(&field.text, 11).is_empty());

        let index_target = Expr::new(
            ExprKind::Index {
                array: Box::new(receiver),
                index: Box::new(call_at(None, "index", vec![], 22)),
            },
            OriginSet::new(Origin::direct(24)),
        )
        .presenting(Type::Int);
        let index_increment = Expr::new(
            ExprKind::PostIncrement {
                target: Box::new(index_target),
            },
            OriginSet::new(Origin::direct(25)),
        );
        let (index, index_map) = emitted_value(index_increment);
        assert!(
            index.text.contains("(left() + right())[index()]++;"),
            "the postfix node must preserve the grouped array receiver and index:\n{}",
            index.text
        );
        assert!(
            index_map
                .text_of_bci(&index.text, 24)
                .contains(&"(left() + right())[index()]")
        );
        assert!(
            index_map
                .text_of_bci(&index.text, 25)
                .contains(&"(left() + right())[index()]++")
        );
        assert!(!index_map.text_of_bci(&index.text, 22).is_empty());
    }

    /// The operator positions of the same scale: `!` binds tighter than every binary operator, so a
    /// binary operand keeps its group; and a `!` in a receiver position does too, because `!a.f()`
    /// reads as `!(a.f())`.
    #[test]
    fn a_prefix_position_and_a_not_receiver_keep_their_groups() {
        let (not, _map) = emitted_value(Expr::direct(
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

        let (not_receiver, _map) = emitted_value(call_at(
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

    #[test]
    fn instanceof_uses_relational_precedence_under_not() {
        let test = Expr::direct(
            ExprKind::InstanceOf {
                value: Box::new(local_at("arg0", 2)),
                ty: "java.lang.String".to_string(),
            },
            3,
        );
        let (plain, _) = emitted_value(test.clone());
        assert!(
            plain.text.contains("arg0 instanceof java.lang.String;"),
            "{}",
            plain.text
        );
        let (negated, _) = emitted_value(Expr::direct(
            ExprKind::Not {
                value: Box::new(test),
            },
            4,
        ));
        assert!(
            negated
                .text
                .contains("!(arg0 instanceof java.lang.String);"),
            "{}",
            negated.text
        );
    }

    #[test]
    fn a_negation_keeps_nested_groups_and_negative_literals_lexically_separate() {
        let (nested, _map) = emitted_value(Expr::direct(
            ExprKind::Neg {
                value: Box::new(Expr::direct(
                    ExprKind::Neg {
                        value: Box::new(local_at("arg0", 2)),
                    },
                    3,
                )),
            },
            4,
        ));
        assert!(
            nested.text.contains("-(-arg0);"),
            "nested negation must not become decrement syntax:\n{}",
            nested.text
        );

        let (literal, _map) = emitted_value(Expr::direct(
            ExprKind::Neg {
                value: Box::new(Expr::direct(ExprKind::Integer(-1), 6)),
            },
            7,
        ));
        assert!(
            literal.text.contains("-(-1);"),
            "a negative integer literal must stay separate from the outer minus:\n{}",
            literal.text
        );

        let (long_literal, _map) = emitted_value(Expr::direct(
            ExprKind::Neg {
                value: Box::new(Expr::direct(ExprKind::Long(-1), 8)),
            },
            9,
        ));
        assert!(
            long_literal.text.contains("-(-1L);"),
            "a negative long literal must stay separate from the outer minus:\n{}",
            long_literal.text
        );

        // A float literal's sign is one of its own bits, so a negation over a signed constant is
        // the same lexical shape: `-` `-0x…` would read as decrement without the group.
        let (float_literal, _map) = emitted_value(Expr::direct(
            ExprKind::Neg {
                value: Box::new(Expr::direct(ExprKind::Float(0x8000_0000), 10)),
            },
            11,
        ));
        assert!(
            float_literal.text.contains("-(-0x0.000000p-126f);"),
            "a negative zero literal must stay separate from the outer minus:\n{}",
            float_literal.text
        );

        let (double_literal, _map) = emitted_value(Expr::direct(
            ExprKind::Neg {
                value: Box::new(Expr::direct(ExprKind::Double(0xbff0_0000_0000_0000), 12)),
            },
            13,
        ));
        assert!(
            double_literal.text.contains("-(-0x1.0000000000000p0d);"),
            "a negative double literal must stay separate from the outer minus:\n{}",
            double_literal.text
        );
    }

    /// A finite float/double literal is spelled from its own bits in the hexadecimal
    /// floating-point form, which round-trips bit for bit: signed zero, the subnormals, the
    /// minimal normal, the maximum and ordinary values keep every field the class file stated,
    /// and the `f`/`d` suffix is always written.
    #[test]
    fn floating_literals_are_spelled_exactly_from_their_own_bits() {
        let spell_f = |bits| spell_float(bits);
        let spell_d = |bits| spell_double(bits);

        // Zero keeps its sign as its own minus; the encoding's zero exponent field writes the
        // subnormal form at the minimal normal exponent.
        assert_eq!(spell_f(0), "0x0.000000p-126f");
        assert_eq!(spell_f(0x8000_0000), "-0x0.000000p-126f");
        assert_eq!(spell_d(0), "0x0.0000000000000p-1022d");
        assert_eq!(spell_d(0x8000_0000_0000_0000), "-0x0.0000000000000p-1022d");

        // The ordinary and boundary values the fixture returns, each with the exact fields its
        // bits state: the 24/53 significand bits split into one integer digit and six/thirteen
        // fractional ones, the exponent at its own bias.
        assert_eq!(spell_f(0x3f80_0000), "0x1.000000p0f");
        assert_eq!(spell_f(0x4000_0000), "0x1.000000p1f");
        assert_eq!(spell_f(0xbf80_0000), "-0x1.000000p0f");
        assert_eq!(spell_f(0x3dcc_cccd), "0x1.99999ap-4f");
        assert_eq!(spell_f(0x3e80_0000), "0x1.000000p-2f");
        assert_eq!(spell_f(0x0000_0001), "0x0.000002p-126f");
        assert_eq!(spell_f(0x0080_0000), "0x1.000000p-126f");
        assert_eq!(spell_f(0x7f7f_ffff), "0x1.fffffep127f");
        assert_eq!(spell_d(0x3ff0_0000_0000_0000), "0x1.0000000000000p0d");
        assert_eq!(spell_d(0x4000_0000_0000_0000), "0x1.0000000000000p1d");
        assert_eq!(spell_d(0xbff0_0000_0000_0000), "-0x1.0000000000000p0d");
        assert_eq!(spell_d(0x3fb9_9999_9999_999a), "0x1.999999999999ap-4d");
        assert_eq!(spell_d(0x3fd0_0000_0000_0000), "0x1.0000000000000p-2d");
        assert_eq!(spell_d(0x0000_0000_0000_0001), "0x0.0000000000001p-1022d");
        assert_eq!(spell_d(0x0010_0000_0000_0000), "0x1.0000000000000p-1022d");
        assert_eq!(spell_d(0x7fef_ffff_ffff_ffff), "0x1.fffffffffffffp1023d");
    }

    /// A concatenation chain's parts: the first `+` starts a **string** concatenation when the first
    /// part is not already a `String`, and a part that is itself an addition keeps its own group in
    /// the right-hand position — which is what makes two parts and one addition-part two texts.
    #[test]
    fn a_concatenation_starts_in_a_string_context_and_keeps_its_parts_groups() {
        let part = |parameter: Type, value: Expr| ConcatPart::new(parameter, value);
        let string = || Type::Reference("java.lang.String".to_string());
        let bang = || Expr::direct(ExprKind::Str("!".to_string()), 15);

        // Two parts that each need conversion, then a `String` part: the empty string starts the
        // text, so neither part is added to the other as a number.
        let (convertible, _map) = emitted_value(Expr::direct(
            ExprKind::Concat {
                parts: vec![
                    part(Type::Int, local_at("arg0", 7)),
                    part(Type::Int, local_at("arg1", 11)),
                    part(string(), bang()),
                ],
            },
            20,
        ));
        assert!(
            convertible.text.contains("\"\" + arg0 + arg1 + \"!\";"),
            "the first `+` is a string concatenation:\n{}",
            convertible.text
        );

        // **One** part that is itself an addition: it keeps its own group, so the sum is evaluated
        // first and converted once — the other program of the same input.
        let (sum_part, _map) = emitted_value(Expr::direct(
            ExprKind::Concat {
                parts: vec![
                    part(
                        Type::Int,
                        sum_at(local_at("arg0", 7), local_at("arg1", 8), 9),
                    ),
                    part(string(), bang()),
                ],
            },
            18,
        ));
        assert!(
            sum_part.text.contains("\"\" + (arg0 + arg1) + \"!\";"),
            "a part that is an addition keeps its own group:\n{}",
            sum_part.text
        );

        // A chain that is already in a string context gains nothing — and an addition part after it
        // still keeps its group, or `"x" + arg0 + arg1` would be two parts rather than one sum.
        let (already, _map) = emitted_value(Expr::direct(
            ExprKind::Concat {
                parts: vec![
                    part(string(), Expr::direct(ExprKind::Str("x".to_string()), 7)),
                    part(
                        Type::Int,
                        sum_at(local_at("arg0", 11), local_at("arg1", 12), 13),
                    ),
                ],
            },
            14,
        ));
        assert!(
            already.text.contains("\"x\" + (arg0 + arg1);"),
            "no decoration, and the sum keeps its own group:\n{}",
            already.text
        );

        // The all-`String` control: every part is a primary, so the text is the parts and their `+`s.
        let (plain, _map) = emitted_value(Expr::direct(
            ExprKind::Concat {
                parts: vec![
                    part(string(), local_at("arg0", 7)),
                    part(string(), local_at("arg1", 11)),
                ],
            },
            18,
        ));
        assert!(
            plain.text.contains("arg0 + arg1;") && !plain.text.contains("\"\""),
            "a chain already in a string context gains nothing:\n{}",
            plain.text
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
        let (emitted, _map) = emitted_value(call_at(
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
        let (emitted, _map) = emitted_value(call_at(
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

        let (index, _map) = emitted_value(Expr::new(
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

        let (returned, _map) = emitted_stmt(
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

        let (thrown, thrown_map) = emitted_stmt(
            StmtKind::Throw {
                value: local_at("arg0", 2),
            },
            &[1],
        );
        assert!(
            thrown.text.contains("throw arg0;"),
            "a throw value is the whole expression to the `;`:\n{}",
            thrown.text
        );
        assert_eq!(
            thrown_map.text_of_bci(&thrown.text, 1),
            vec!["    throw arg0;\n"],
            "the throw statement keeps its own source anchor"
        );
        assert_eq!(
            thrown_map.text_of_bci(&thrown.text, 2),
            vec!["arg0"],
            "the thrown expression keeps its producer anchor"
        );

        let (declared, _map) = emitted_stmt(
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

        let (conditional, _map) = emitted_stmt(
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

        let (switch, _map) = emitted_stmt(
            StmtKind::Switch {
                value: sum_at(local_at("arg0", 2), local_at("arg1", 3), 4),
                arms: vec![crate::ast::SwitchArm {
                    keys: vec![0],
                    labels: None,
                    default: false,
                    fall_through: false,
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

        let (throwing_switch, _map) = emitted_stmt(
            StmtKind::Switch {
                value: local_at("arg0", 2),
                arms: vec![crate::ast::SwitchArm {
                    keys: vec![0],
                    labels: None,
                    default: false,
                    fall_through: false,
                    body: vec![Stmt::new(
                        StmtKind::Throw {
                            value: Expr::direct(ExprKind::Null, 6),
                        },
                        OriginSet::new(Origin::direct(6)),
                    )],
                }],
            },
            &[4],
        );
        assert!(
            throwing_switch.text.contains("throw null;"),
            "a switch arm keeps its throwing terminator:\n{}",
            throwing_switch.text
        );
        assert!(
            !throwing_switch.text.contains("break;"),
            "a throwing switch arm does not gain an unreachable break:\n{}",
            throwing_switch.text
        );

        let (exiting_switch, _map) = emitted_stmt(
            StmtKind::Switch {
                value: local_at("arg0", 2),
                arms: vec![crate::ast::SwitchArm {
                    keys: vec![0],
                    labels: None,
                    default: false,
                    fall_through: false,
                    body: vec![Stmt::new(
                        StmtKind::If {
                            cond: local_at("arg1", 5),
                            then_body: vec![Stmt::new(
                                StmtKind::Break {
                                    label: Some("outer".to_owned()),
                                },
                                OriginSet::new(Origin::direct(6)),
                            )],
                            else_body: vec![Stmt::new(
                                StmtKind::Continue { label: None },
                                OriginSet::new(Origin::direct(7)),
                            )],
                        },
                        OriginSet::new(Origin::direct(5)),
                    )],
                }],
            },
            &[2],
        );
        assert!(exiting_switch.text.contains("break outer;"));
        assert!(exiting_switch.text.contains("continue;"));
        assert!(
            !exiting_switch
                .text
                .lines()
                .any(|line| line.trim() == "break;"),
            "both paths leave the switch arm, so a trailing switch break is unreachable:\n{}",
            exiting_switch.text
        );

        let (partly_exiting_switch, _map) = emitted_stmt(
            StmtKind::Switch {
                value: local_at("arg0", 2),
                arms: vec![crate::ast::SwitchArm {
                    keys: vec![0],
                    labels: None,
                    default: false,
                    fall_through: false,
                    body: vec![Stmt::new(
                        StmtKind::If {
                            cond: local_at("arg1", 5),
                            then_body: vec![Stmt::new(
                                StmtKind::Break {
                                    label: Some("outer".to_owned()),
                                },
                                OriginSet::new(Origin::direct(6)),
                            )],
                            else_body: Vec::new(),
                        },
                        OriginSet::new(Origin::direct(5)),
                    )],
                }],
            },
            &[2],
        );
        assert!(
            partly_exiting_switch
                .text
                .lines()
                .any(|line| line.trim() == "break;"),
            "the path that completes normally still needs the switch break:\n{}",
            partly_exiting_switch.text
        );

        let (lock, _map) = emitted_stmt(
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

        let (lambda_body, _map) = emitted_value(Expr::direct(
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
        let (plain, _map) = emitted_value(call_at(Some(local_at("arg0", 2)), "foo", vec![], 3));
        assert!(
            plain.text.contains("arg0.foo();"),
            "a name receiver is a primary:\n{}",
            plain.text
        );

        let (chained, _map) = emitted_value(call_at(
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

        let (tighter, _map) = emitted_value(sum_at(
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

        let (left, _map) = emitted_value(sum_at(
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

        let (not_operand, _map) = emitted_value(Expr::direct(
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

    #[test]
    fn bitwise_operators_keep_java_precedence_and_mixed_grouping() {
        let expression = binary_at(
            BinaryOp::BitwiseXor,
            binary_at(
                BinaryOp::BitwiseOr,
                local_at("arg0", 2),
                local_at("arg1", 3),
                4,
            ),
            binary_at(
                BinaryOp::BitwiseAnd,
                local_at("arg2", 5),
                sum_at(local_at("arg3", 6), local_at("arg4", 7), 8),
                9,
            ),
            10,
        );
        let (emitted, _map) = emitted_value(expression);
        assert!(
            emitted.text.contains("(arg0 | arg1) ^ arg2 & arg3 + arg4;"),
            "`&` binds more tightly than `^`, which binds more tightly than `|`, while `+`\n\
             binds more tightly than bitwise operators:\n{}",
            emitted.text
        );

        let additive = binary_at(
            BinaryOp::Add,
            local_at("arg0", 11),
            binary_at(
                BinaryOp::BitwiseOr,
                local_at("arg1", 12),
                local_at("arg2", 13),
                14,
            ),
            15,
        );
        let (emitted, _map) = emitted_value(additive);
        assert!(
            emitted.text.contains("arg0 + (arg1 | arg2);"),
            "a lower-precedence bitwise child of `+` keeps its required parentheses:\n{}",
            emitted.text
        );
    }

    #[test]
    fn shifts_keep_precedence_associativity_casts_and_primary_operands() {
        let shift = |op, left, right, at| binary_at(op, left, right, at);
        let nested_right = shift(
            BinaryOp::LeftShift,
            local_at("arg0", 2),
            shift(
                BinaryOp::UnsignedRightShift,
                local_at("arg1", 3),
                local_at("arg2", 4),
                5,
            ),
            6,
        );
        assert!(
            emitted_value(nested_right)
                .0
                .text
                .contains("arg0 << (arg1 >>> arg2);")
        );

        let nested_left = shift(
            BinaryOp::UnsignedRightShift,
            shift(
                BinaryOp::LeftShift,
                local_at("arg0", 2),
                local_at("arg1", 3),
                4,
            ),
            local_at("arg2", 5),
            6,
        );
        assert!(
            emitted_value(nested_left)
                .0
                .text
                .contains("arg0 << arg1 >>> arg2;")
        );

        let grouped_add = sum_at(
            local_at("arg0", 2),
            shift(
                BinaryOp::LeftShift,
                local_at("arg1", 3),
                local_at("arg2", 4),
                5,
            ),
            6,
        );
        assert!(
            emitted_value(grouped_add)
                .0
                .text
                .contains("arg0 + (arg1 << arg2);")
        );

        let multiplied = shift(
            BinaryOp::RightShift,
            binary_at(
                BinaryOp::Multiply,
                local_at("arg0", 2),
                local_at("arg1", 3),
                4,
            ),
            sum_at(local_at("arg2", 5), local_at("arg3", 6), 7),
            8,
        );
        assert!(
            emitted_value(multiplied)
                .0
                .text
                .contains("arg0 * arg1 >> arg2 + arg3;")
        );

        let cast = Expr::direct(
            ExprKind::Cast {
                ty: Type::Long,
                value: Box::new(local_at("arg0", 2)),
            },
            3,
        );
        let width = shift(BinaryOp::LeftShift, cast, local_at("arg1", 4), 5);
        assert!(emitted_value(width).0.text.contains("(long) arg0 << arg1;"));

        let calls = shift(
            BinaryOp::LeftShift,
            call_at(None, "left", vec![], 2),
            call_at(None, "right", vec![], 3),
            4,
        );
        assert!(emitted_value(calls).0.text.contains("left() << right();"));

        let relation = binary_at(
            BinaryOp::Less,
            shift(
                BinaryOp::LeftShift,
                local_at("arg0", 2),
                local_at("arg1", 3),
                4,
            ),
            local_at("arg2", 5),
            6,
        );
        let equality = binary_at(BinaryOp::Equal, relation, local_at("arg3", 7), 8);
        let bitwise = binary_at(BinaryOp::BitwiseAnd, equality, local_at("arg4", 9), 10);
        assert!(
            emitted_value(bitwise)
                .0
                .text
                .contains("arg0 << arg1 < arg2 == arg3 & arg4;")
        );
    }
}
