//! An **independent** small-graph oracle for the four things a presentation must not get wrong:
//! a branch's polarity, a loop's per-iteration effects, a method's `return`, and which exception
//! record a throw site reaches first.
//!
//! # What "independent" means here, and what it does not
//!
//! The oracle holds its **own** model of the bytecode (a naive stack machine over a hand-written
//! listing, decoded by its own opcode match) and its **own** model of Java (a naive parser for the
//! handful of statement shapes the subset writes, and an evaluator for their expressions). It
//! compares the two:
//!
//! * the bytecode model says what the fixture *does* — which calls happen how often, in which order,
//!   how many times a test is evaluated, what the method returns;
//! * the artifact model says what the *recovered text* does — parsed from the text alone;
//! * the comparator runs both on the same fixture and requires the same observations.
//!
//! So a flipped `ifeq`/`ifne`, a condition hoisted out of a loop, a `return` of the wrong value or a
//! loop whose body runs one time too many all show up as a difference between the two. What the
//! oracle is *not* independent of is the fixture's bytes (shared ground truth, which is the point)
//! and the four opcodes' meanings, which it decodes itself.
//!
//! The model section below is fenced by two markers and a test asserts that it names **none** of the
//! recovery layers' own helpers — no `Region`, no `StmtKind`, no `NormalFlowView`, no segment table:
//! if the oracle ever grew a call into the thing it is checking, that guard goes red.
//!
//! # Why it lives under `#[cfg(test)]`
//!
//! Nothing here is part of a production run: it is a second opinion, compiled only for the tests of
//! this crate, and it never enters the artifact's own path.

// ------------------------------- oracle model -------------------------------
// Everything between these markers is the oracle's own: a naive machine for the fixture's bytes, a
// naive parser and evaluator for the produced text, and the comparator that requires them to agree.

/// One instruction of the oracle's own model of a fixture.
///
/// The model is deliberately tiny: it holds exactly the instructions the oracle's fixtures use, and
/// the decoder **panics** on anything else. A fixture that grew an unmodelled instruction fails
/// loudly instead of being silently misread — which is the difference between an oracle and a
/// second guess.
#[derive(Clone, Debug, PartialEq)]
enum Instr {
    /// A constant the opcode or its operand states (`aconst_null` is the null reference, which this
    /// model spells as the value zero it is never compared against).
    Const(i64),
    Load(u16),
    Store(u16),
    Add,
    Sub,
    /// A void call: it pops its receiver and its arguments and records one effect.
    Invoke {
        pops: usize,
        args: usize,
    },
    Goto(u32),
    /// A one-value conditional transfer: pop a value, compare it against zero, transfer if it holds.
    Branch {
        cmp: Cmp,
        target: u32,
    },
    /// `ireturn`/`return`: pop the value when the method returns one.
    Return {
        value: bool,
    },
}

/// The comparisons the oracle's fixtures branch on.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Cmp {
    Eq,
    Ne,
    Lt,
    Gt,
}

/// What one run did: the effects, in order, and the value it left.
#[derive(Clone, Debug, Default, PartialEq)]
struct Trace {
    /// One entry per call, `name(arguments)`.
    calls: Vec<String>,
    /// How many times a test was evaluated (a branch executed, a condition read).
    tests: usize,
    /// The value the method returned, when it returned one.
    returned: Option<i64>,
}

/// The oracle's decoder: opcode bytes to the model above, with the BCIs they start at.
fn decode(code: &[u8]) -> Vec<(u32, usize, Instr)> {
    let mut instructions = Vec::new();
    let mut at = 0usize;
    while at < code.len() {
        let bci = at as u32;
        let opcode = code[at];
        let instruction = match opcode {
            0x01 => Instr::Const(0),                               // aconst_null
            0x02 => Instr::Const(-1),                              // iconst_m1
            0x03..=0x08 => Instr::Const(i64::from(opcode) - 3),    // iconst_0..iconst_5
            0x09 | 0x0a => Instr::Const(i64::from(opcode) - 0x09), // lconst_0/lconst_1
            0x10 => Instr::Const(i64::from(code[at + 1] as i8)),   // bipush
            0x11 => Instr::Const(i64::from(i16::from_be_bytes([code[at + 1], code[at + 2]]))), // sipush
            0x15 => Instr::Load(u16::from(code[at + 1])), // iload
            0x19 => Instr::Load(u16::from(code[at + 1])), // aload
            0x1a..=0x1d => Instr::Load(u16::from(opcode - 0x1a)), // iload_0..iload_3
            0x2a..=0x2d => Instr::Load(u16::from(opcode - 0x2a)), // aload_0..aload_3
            0x36 => Instr::Store(u16::from(code[at + 1])), // istore
            0x3a => Instr::Store(u16::from(code[at + 1])), // astore
            0x3b..=0x3e => Instr::Store(u16::from(opcode - 0x3b)), // istore_0..istore_3
            0x4b..=0x4e => Instr::Store(u16::from(opcode - 0x4b)), // astore_0..astore_3
            0x60 => Instr::Add,
            0x64 => Instr::Sub,
            0x99 => Instr::Branch {
                cmp: Cmp::Eq,
                target: target(code, at),
            }, // ifeq
            0x9a => Instr::Branch {
                cmp: Cmp::Ne,
                target: target(code, at),
            }, // ifne
            0x9b => Instr::Branch {
                cmp: Cmp::Lt,
                target: target(code, at),
            }, // iflt
            0x9d => Instr::Branch {
                cmp: Cmp::Gt,
                target: target(code, at),
            }, // ifgt
            0xa7 => Instr::Goto(target(code, at)),      // goto
            0xb9 => Instr::Invoke { pops: 2, args: 1 }, // invokeinterface, the fixtures' run:(J)V
            0xac => Instr::Return { value: true },      // ireturn
            0xb1 => Instr::Return { value: false },     // return
            other => panic!("the oracle does not model the opcode {other:#04x} at BCI {bci}"),
        };
        instructions.push((bci, width(opcode), instruction));
        at += width(opcode);
    }
    instructions
}

/// How many bytes one modelled instruction occupies.
fn width(opcode: u8) -> usize {
    match opcode {
        0x01..=0x0a
        | 0x1a..=0x1d
        | 0x2a..=0x2d
        | 0x3b..=0x3e
        | 0x4b..=0x4e
        | 0x60
        | 0x64
        | 0xac
        | 0xb1 => 1,
        0x10 | 0x15 | 0x19 | 0x36 | 0x3a => 2,
        0x11 => 3,
        0x99 | 0x9a | 0x9b | 0x9d | 0xa7 => 3,
        0xb9 => 5,
        other => panic!("the oracle does not model the opcode {other:#04x}"),
    }
}

/// One branch's absolute target: the offset is relative to the instruction's own BCI, and the
/// instruction's own operand starts two bytes in.
fn target(code: &[u8], at: usize) -> u32 {
    let offset = i16::from_be_bytes([code[at + 1], code[at + 2]]);
    ((at as i64) + i64::from(offset)) as u32
}

/// Runs the oracle's model of one fixture from its first instruction.
fn run_bytecode(code: &[u8]) -> Trace {
    let instructions: std::collections::BTreeMap<u32, (usize, Instr)> = decode(code)
        .into_iter()
        .map(|(bci, width, instruction)| (bci, (width, instruction)))
        .collect();
    let mut trace = Trace::default();
    let mut locals: std::collections::BTreeMap<u16, i64> = std::collections::BTreeMap::new();
    let mut stack: Vec<i64> = Vec::new();
    let mut bci = 0u32;
    let mut steps = 0usize;
    loop {
        steps += 1;
        assert!(steps < 10_000, "the fixture does not terminate");
        let Some((width, instruction)) = instructions.get(&bci).cloned() else {
            panic!("the fixture reaches BCI {bci}, which holds no instruction");
        };
        match instruction {
            Instr::Const(value) => {
                stack.push(value);
                bci += width as u32;
            }
            Instr::Load(slot) => {
                let value = *locals
                    .get(&slot)
                    .unwrap_or_else(|| panic!("the fixture reads local {slot} before writing it"));
                stack.push(value);
                bci += width as u32;
            }
            Instr::Store(slot) => {
                let value = stack.pop().expect("a store pops a value");
                locals.insert(slot, value);
                bci += width as u32;
            }
            Instr::Add | Instr::Sub => {
                let right = stack.pop().expect("a binary operator pops two values");
                let left = stack.pop().expect("a binary operator pops two values");
                stack.push(if instruction == Instr::Add {
                    left + right
                } else {
                    left - right
                });
                bci += width as u32;
            }
            Instr::Invoke { pops, args } => {
                let mut values = Vec::new();
                for _ in 0..pops {
                    values.push(stack.pop().expect("a call pops its receiver and arguments"));
                }
                values.reverse();
                let arguments: Vec<String> = values
                    .iter()
                    .skip(pops - args)
                    .map(|value| value.to_string())
                    .collect();
                trace.calls.push(format!("run({})", arguments.join(", ")));
                bci += width as u32;
            }
            Instr::Goto(target) => bci = target,
            Instr::Branch { cmp, target } => {
                let value = stack.pop().expect("a branch pops the value it tests");
                trace.tests += 1;
                let transfers = match cmp {
                    Cmp::Eq => value == 0,
                    Cmp::Ne => value != 0,
                    Cmp::Lt => value < 0,
                    Cmp::Gt => value > 0,
                };
                bci = if transfers {
                    target
                } else {
                    bci + width as u32
                };
            }
            Instr::Return { value } => {
                trace.returned = value.then(|| stack.pop().expect("the return pops its value"));
                return trace;
            }
        }
    }
}

/// One statement of the oracle's own model of the produced text, parsed from the text alone.
#[derive(Clone, Debug, PartialEq)]
enum Text {
    Assign(String, Expr),
    Call(Expr),
    Return(Option<Expr>),
    If(Expr, Vec<Text>, Vec<Text>),
    While(Expr, Vec<Text>),
    DoWhile(Expr, Vec<Text>),
}

/// One expression of the oracle's own model of the produced text.
#[derive(Clone, Debug, PartialEq)]
enum Expr {
    Local(String),
    Int(i64),
    Null,
    Call {
        receiver: Option<Box<Expr>>,
        name: String,
        args: Vec<Expr>,
    },
    Binary(Box<Expr>, String, Box<Expr>),
}

/// Parses one produced artifact into the oracle's own statement model.
///
/// The parser knows the shapes this subset writes and **panics** on anything else (a `switch`, a
/// declaration it cannot read): the oracle is a second opinion about a small graph, not a Java
/// front end, and a shape it does not model is a shape it must not silently skip.
fn parse(text: &str) -> Vec<Text> {
    let mut lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect();
    // The envelope: the body's own opening brace and the one that closes it.
    if lines.first() == Some(&"{") {
        lines.remove(0);
    }
    if lines.last() == Some(&"}") {
        lines.pop();
    }
    let (statements, rest) = parse_block(&lines);
    assert!(
        rest.is_empty(),
        "the artifact holds text the oracle did not parse: {rest:?}"
    );
    statements
}

/// Parses statements until the block that contains them ends, and returns what it did not read.
fn parse_block<'a>(lines: &'a [&'a str]) -> (Vec<Text>, &'a [&'a str]) {
    let mut statements = Vec::new();
    let mut rest = lines;
    while let Some((line, tail)) = rest.split_first() {
        let line = *line;
        if line == "}" || line == "} else {" || line.starts_with("} while (") {
            break;
        }
        if let Some(condition) = line.strip_prefix("if (") {
            let condition = parse_expr(condition.trim_end_matches(") {"));
            let (then_body, after_then) = parse_block(tail);
            let (else_body, after_if) = match after_then.split_first() {
                Some((&"} else {", tail)) => {
                    let (else_body, after_else) = parse_block(tail);
                    (else_body, after_else)
                }
                _ => (Vec::new(), after_then),
            };
            statements.push(Text::If(condition, then_body, else_body));
            rest = skip_closing_brace(after_if);
            continue;
        }
        if let Some(condition) = line.strip_prefix("while (") {
            let condition = parse_expr(condition.trim_end_matches(") {"));
            let (body, after) = parse_block(tail);
            statements.push(Text::While(condition, body));
            rest = skip_closing_brace(after);
            continue;
        }
        if line == "do {" {
            let (body, after) = parse_block(tail);
            let Some((test, after_test)) = after.split_first() else {
                panic!("a `do` whose test is not written");
            };
            let Some(condition) = test.strip_prefix("} while (") else {
                panic!("a `do` whose test is not a `while`: {test:?}");
            };
            statements.push(Text::DoWhile(
                parse_expr(condition.trim_end_matches(");")),
                body,
            ));
            rest = after_test;
            continue;
        }
        if let Some(value) = line.strip_prefix("return") {
            let value = value.trim().trim_end_matches(';').trim();
            statements.push(Text::Return((!value.is_empty()).then(|| parse_expr(value))));
            rest = tail;
            continue;
        }
        let statement = line.trim_end_matches(';');
        // A declaration spells its type before the name; both shapes assign one value.
        match statement.split_once(" = ") {
            Some((left, value)) => statements.push(Text::Assign(
                left.split_whitespace().last().unwrap_or(left).to_string(),
                parse_expr(value),
            )),
            None => statements.push(Text::Call(parse_expr(statement))),
        }
        rest = tail;
    }
    (statements, rest)
}

/// The line after a block's own closing brace.
fn skip_closing_brace<'a>(lines: &'a [&'a str]) -> &'a [&'a str] {
    match lines.split_first() {
        Some((&"}", tail)) => tail,
        _ => lines,
    }
}

/// One expression of the produced text, parsed left to right.
///
/// The emitter writes binary expressions flat, one space around each operator, and the oracle's
/// fixtures use single operators per expression — so reading the first operator from the left is the
/// same association the text states, and the fixtures' own assertions keep it that way.
fn parse_expr(text: &str) -> Expr {
    let text = text.trim();
    for operator in [" != ", " >= ", " <= ", " == ", " < ", " > ", " + ", " - "] {
        if let Some(index) = text.find(operator) {
            let (left, rest) = text.split_at(index);
            let right = &rest[operator.len()..];
            return Expr::Binary(
                Box::new(parse_expr(left)),
                operator.trim().to_string(),
                Box::new(parse_expr(right)),
            );
        }
    }
    if let Some(open) = text.find('(') {
        let (callee, arguments) = text.split_at(open);
        let callee = callee.trim_end_matches('.');
        let (receiver, name) = match callee.split_once('.') {
            Some((receiver, name)) => (Some(Box::new(parse_expr(receiver))), name),
            None => (None, callee),
        };
        let arguments = arguments
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .trim_end_matches(';');
        let args = if arguments.is_empty() {
            Vec::new()
        } else {
            arguments.split(", ").map(parse_expr).collect()
        };
        return Expr::Call {
            receiver,
            name: name.to_string(),
            args,
        };
    }
    if text == "null" {
        return Expr::Null;
    }
    if let Ok(value) = text.trim_end_matches('L').parse::<i64>() {
        return Expr::Int(value);
    }
    Expr::Local(text.to_string())
}

/// Evaluates one expression against the locals it reads, recording no effects.
fn evaluate(
    expr: &Expr,
    locals: &std::collections::BTreeMap<String, i64>,
    trace: &mut Trace,
) -> i64 {
    match expr {
        Expr::Local(name) => *locals
            .get(name)
            .unwrap_or_else(|| panic!("the text reads `{name}`, which it never wrote")),
        Expr::Int(value) => *value,
        Expr::Null => 0,
        Expr::Call {
            receiver,
            name,
            args,
        } => {
            if let Some(receiver) = receiver {
                evaluate(receiver, locals, trace);
            }
            let arguments: Vec<String> = args
                .iter()
                .map(|argument| evaluate(argument, locals, trace).to_string())
                .collect();
            trace
                .calls
                .push(format!("{name}({})", arguments.join(", ")));
            // The fixture's call is void; a call whose value the text reads would be a fixture this
            // oracle does not model.
            0
        }
        Expr::Binary(left, operator, right) => {
            let left = evaluate(left, locals, trace);
            let right = evaluate(right, locals, trace);
            match operator.as_str() {
                "+" => left + right,
                "-" => left - right,
                "==" => i64::from(left == right),
                "!=" => i64::from(left != right),
                "<" => i64::from(left < right),
                ">" => i64::from(left > right),
                "<=" => i64::from(left <= right),
                ">=" => i64::from(left >= right),
                other => panic!("the oracle does not model the operator `{other}`"),
            }
        }
    }
}

/// Runs the oracle's model of one parsed artifact, from its first statement.
fn run_text(statements: &[Text]) -> Trace {
    let mut trace = Trace::default();
    let mut locals: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();
    let mut steps = 0usize;
    execute(statements, &mut locals, &mut trace, &mut steps);
    trace
}

/// Executes one block of the oracle's statement model.
fn execute(
    statements: &[Text],
    locals: &mut std::collections::BTreeMap<String, i64>,
    trace: &mut Trace,
    steps: &mut usize,
) {
    for statement in statements {
        *steps += 1;
        assert!(*steps < 10_000, "the artifact does not terminate");
        match statement {
            Text::Assign(name, value) => {
                let value = evaluate(value, locals, trace);
                locals.insert(name.clone(), value);
            }
            Text::Call(expr) => {
                evaluate(expr, locals, trace);
            }
            Text::Return(value) => {
                trace.returned = Some(match value {
                    Some(value) => evaluate(value, locals, trace),
                    None => 0,
                });
                return;
            }
            Text::If(condition, then_body, else_body) => {
                trace.tests += 1;
                let holds = evaluate(condition, locals, trace) != 0;
                execute(
                    if holds { then_body } else { else_body },
                    locals,
                    trace,
                    steps,
                );
            }
            Text::While(condition, body) => loop {
                trace.tests += 1;
                if evaluate(condition, locals, trace) == 0 {
                    break;
                }
                execute(body, locals, trace, steps);
            },
            Text::DoWhile(condition, body) => loop {
                execute(body, locals, trace, steps);
                trace.tests += 1;
                if evaluate(condition, locals, trace) == 0 {
                    break;
                }
            },
        }
    }
}

/// Requires the artifact's model of a fixture to do what the fixture's bytes do.
fn compare(fixture: &[u8], text: &str) -> Result<(), String> {
    let expected = run_bytecode(fixture);
    let actual = run_text(&parse(text));
    if actual.calls != expected.calls {
        return Err(format!(
            "the calls differ: the bytecode makes {:?}, the text makes {:?}",
            expected.calls, actual.calls
        ));
    }
    if actual.returned != expected.returned {
        return Err(format!(
            "the returned value differs: the bytecode returns {:?}, the text returns {:?}",
            expected.returned, actual.returned
        ));
    }
    if actual.tests != expected.tests {
        return Err(format!(
            "the test runs {} time(s) in the bytecode and {} time(s) in the text",
            expected.tests, actual.tests
        ));
    }
    Ok(())
}

/// The record a throw at one BCI reaches first: the first one of the table that covers it, in table
/// order — the priority the JVM states and the only fact a presentation of a crossing table can
/// carry.
fn primary_record(table: &[(u32, u32, u32, u32)], throw_bci: u32) -> Option<u32> {
    table
        .iter()
        .enumerate()
        .find(|(_, (start, end, _, _))| *start <= throw_bci && throw_bci < *end)
        .map(|(ordinal, _)| ordinal as u32)
}

// ----------------------------- end oracle model -----------------------------

// ------------------------------- the fixtures -------------------------------

/// `int local1 = <opcode>; if (<branch>) { local1 = 1; } else { local1 = 0; } return local1;`
///
/// The one shape a polarity mistake shows up in: which arm runs decides the returned value, and the
/// branch's sense decides which arm that is. Both senses are built from the same layout, so a
/// presentation that reads the wrong one is caught whichever way round it went.
fn polarity_fixture(initial: u8, sense: u8) -> Vec<u8> {
    vec![
        initial, // 0: iconst
        0x3c,    // 1: istore_1
        0x1b,    // 2: iload_1
        sense, 0x00, 0x08, // 3: ifeq/ifne 11
        0x04, // 6: iconst_1     (the fall-through arm)
        0x3c, // 7: istore_1
        0xa7, 0x00, 0x05, // 8: goto 13
        0x03, // 11: iconst_0    (the arm the branch transfers to)
        0x3c, // 12: istore_1
        0x1b, // 13: iload_1
        0xac, // 14: ireturn
    ]
}

/// `int local1 = <opcode>; Runnable local2 = null; while (local1 >= 0) { local2.run(0L); local1 =
/// local1 - 1; } return 7;`
///
/// The loop's effect count is the fixture's whole point: the body's call happens once per iteration,
/// the header's test once per iteration *and once more* when it finally fails.
fn while_fixture(initial: u8) -> Vec<u8> {
    vec![
        initial, // 0: iconst
        0x3c,    // 1: istore_1
        0x01,    // 2: aconst_null
        0x4d,    // 3: astore_2
        0x1b,    // 4: iload_1      (the header)
        0x9b, 0x00, 0x11, // 5: iflt 22
        0x2c, // 8: aload_2
        0x09, // 9: lconst_0
        0xb9, 0x00, 0x0f, 0x02, 0x00, // 10: invokeinterface #15
        0x1b, // 15: iload_1
        0x04, // 16: iconst_1
        0x64, // 17: isub
        0x3c, // 18: istore_1
        0xa7, 0xff, 0xf1, // 19: goto 4
        0x10, 0x07, // 22: bipush 7
        0xac, // 24: ireturn
    ]
}

/// `int local1 = <opcode>; Runnable local2 = null; do { local2.run(0L); local1 = local1 - 1; } while
/// (local1 > 0); return 9;`
///
/// The bottom-tested shape: the body runs before the test is read, so its effect count is one higher
/// than a `while` with the same condition would run.
fn do_while_fixture(initial: u8) -> Vec<u8> {
    vec![
        initial, // 0: iconst
        0x3c,    // 1: istore_1
        0x01,    // 2: aconst_null
        0x4d,    // 3: astore_2
        0x2c,    // 4: aload_2     (the header: the branch's target)
        0x09,    // 5: lconst_0
        0xb9, 0x00, 0x0f, 0x02, 0x00, // 6: invokeinterface #15
        0x1b, // 11: iload_1
        0x04, // 12: iconst_1
        0x64, // 13: isub
        0x3c, // 14: istore_1
        0x1b, // 15: iload_1   (the latch)
        0x9d, 0xff, 0xf4, // 16: ifgt 4
        0x10, 0x09, // 19: bipush 9
        0xac, // 21: ireturn
    ]
}

/// The two records of the crossing table the exception check reads, as `(start, end, ordinal,
/// catch_type)`: `[2, 4)` catching `RuntimeException` and `[3, 6)` catching `Error`.
///
/// They cross — neither contains the other — and both cover the `athrow` at BCI 3, so which handler
/// a throw there reaches is the table's order alone.
const CROSSING_TABLE: [(u32, u32, u32, u32); 2] = [(2, 4, 0, 9), (3, 6, 1, 11)];

// ------------------------------- the harness --------------------------------

use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::test_class;
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::LoaderId;
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

use crate::facts::{MethodFacts, RecoveryFacts};
use crate::report::{RecoveryReport, RecoveryRequest, recover};

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

/// The analyzed payload of the one method of one assembled class.
fn analyze(class: &[u8]) -> jarde_jvm::method_ir::MethodIrAnalysis {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("the fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(b"method".to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Snapshot {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let environment = ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: domain.clone(),
        },
        domains: vec![domain],
        providers: Vec::new(),
    };
    let request = MethodAnalysisRequest {
        environment,
        method,
        stages: AnalysisStage::ALL.to_vec(),
    };
    analyze_method_ir(&[snapshot], &request, &mut budget)
        .expect("the analysis of an assembled body runs")
}

/// The recovery report of one assembled body, with no debug names.
fn recovery_of(code: &[u8], max_locals: u16) -> RecoveryReport {
    let class = test_class::single_method(52, 4, max_locals, code);
    let analysis = analyze(&class);
    let facts = RecoveryFacts::new(MethodFacts::new("method", "()V", 0));
    let mut budget = Budget::new(limits());
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, crate::pass::JAVA_8),
        &mut budget,
    )
}

/// A class file whose exception table is [`CROSSING_TABLE`], with an `athrow` inside both records.
///
/// The body is `aconst_null; astore_0; aload_0; athrow; return`, and the two handler entries are two
/// more `athrow`s at BCI 5 and 6.
fn crossing_class() -> Vec<u8> {
    let mut pool: Vec<u8> = Vec::new();
    let utf8 = |pool: &mut Vec<u8>, text: &str| {
        pool.push(1);
        pool.extend_from_slice(&(text.len() as u16).to_be_bytes());
        pool.extend_from_slice(text.as_bytes());
    };
    let class = |pool: &mut Vec<u8>, name_index: u16| {
        pool.push(7);
        pool.extend_from_slice(&name_index.to_be_bytes());
    };
    utf8(&mut pool, "Test"); // 1
    class(&mut pool, 1); // 2
    utf8(&mut pool, "java/lang/Object"); // 3
    class(&mut pool, 3); // 4
    utf8(&mut pool, "method"); // 5
    utf8(&mut pool, "()V"); // 6
    utf8(&mut pool, "Code"); // 7
    utf8(&mut pool, "java/lang/RuntimeException"); // 8
    class(&mut pool, 8); // 9
    utf8(&mut pool, "java/lang/Error"); // 10
    class(&mut pool, 10); // 11

    let code: Vec<u8> = vec![
        0x01, // 0: aconst_null
        0x4b, // 1: astore_0
        0x2a, // 2: aload_0
        0xbf, // 3: athrow
        0xb1, // 4: return
        0xbf, // 5: athrow
        0xbf, // 6: athrow
    ];
    let mut attribute = Vec::new();
    attribute.extend_from_slice(&1_u16.to_be_bytes()); // max_stack
    attribute.extend_from_slice(&1_u16.to_be_bytes()); // max_locals
    attribute.extend_from_slice(&(code.len() as u32).to_be_bytes());
    attribute.extend_from_slice(&code);
    attribute.extend_from_slice(&2_u16.to_be_bytes());
    for (start, end, ordinal, catch) in CROSSING_TABLE {
        let _ = ordinal;
        let handler = 5 + ordinal as u16;
        attribute.extend_from_slice(&(start as u16).to_be_bytes());
        attribute.extend_from_slice(&(end as u16).to_be_bytes());
        attribute.extend_from_slice(&handler.to_be_bytes());
        attribute.extend_from_slice(&(catch as u16).to_be_bytes());
    }
    attribute.extend_from_slice(&0_u16.to_be_bytes());

    let mut out = Vec::new();
    out.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes());
    out.extend_from_slice(&52_u16.to_be_bytes());
    out.extend_from_slice(&12_u16.to_be_bytes());
    out.extend_from_slice(&pool);
    out.extend_from_slice(&0x0021_u16.to_be_bytes());
    out.extend_from_slice(&2_u16.to_be_bytes());
    out.extend_from_slice(&4_u16.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes());
    out.extend_from_slice(&1_u16.to_be_bytes());
    out.extend_from_slice(&0x0009_u16.to_be_bytes());
    out.extend_from_slice(&5_u16.to_be_bytes());
    out.extend_from_slice(&6_u16.to_be_bytes());
    out.extend_from_slice(&1_u16.to_be_bytes());
    out.extend_from_slice(&7_u16.to_be_bytes());
    out.extend_from_slice(&(attribute.len() as u32).to_be_bytes());
    out.extend_from_slice(&attribute);
    out.extend_from_slice(&0_u16.to_be_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The inputs the oracle tries for every fixture: negative, zero and positive, so an arm and a
    /// loop count both have to be right.
    const INPUTS: [u8; 3] = [0x02, 0x03, 0x06]; // iconst_m1, iconst_0, iconst_3

    #[test]
    fn the_oracle_and_the_presentation_agree_on_polarity() {
        // Both senses, three inputs each: the two fixtures differ only in the branch's own byte.
        for sense in [0x99_u8, 0x9a] {
            for initial in INPUTS {
                let fixture = polarity_fixture(initial, sense);
                let report = recovery_of(&fixture, 2);
                assert!(report.produced(), "{:?}", report.outcome);
                compare(&fixture, &report.text).unwrap_or_else(|error| {
                    panic!(
                        "sense {sense:#04x}, input {initial:#04x}: {error}\n{}",
                        report.text
                    )
                });
            }
        }
    }

    #[test]
    fn the_oracle_and_the_presentation_agree_on_a_loops_effects() {
        for initial in INPUTS {
            let fixture = while_fixture(initial);
            let report = recovery_of(&fixture, 3);
            assert!(report.produced(), "{:?}", report.outcome);
            let expected = run_bytecode(&fixture);
            assert!(
                expected.tests >= 1,
                "the fixture's own test ran at least once"
            );
            compare(&fixture, &report.text)
                .unwrap_or_else(|error| panic!("input {initial:#04x}: {error}\n{}", report.text));
        }
    }

    #[test]
    fn the_oracle_and_the_presentation_agree_on_a_do_whiles_effects() {
        for initial in INPUTS {
            let fixture = do_while_fixture(initial);
            let report = recovery_of(&fixture, 3);
            assert!(report.produced(), "{:?}", report.outcome);
            let expected = run_bytecode(&fixture);
            assert_eq!(
                expected.calls.len(),
                expected.tests,
                "the body runs before every test of this fixture"
            );
            compare(&fixture, &report.text)
                .unwrap_or_else(|error| panic!("input {initial:#04x}: {error}\n{}", report.text));
        }
    }

    #[test]
    fn the_oracle_rejects_a_flipped_condition() {
        // The mutation the whole check exists for: the same text with the branch's sense turned
        // around. The oracle has to see it, on the inputs where the two arms differ.
        let fixture = polarity_fixture(0x03, 0x9a);
        let report = recovery_of(&fixture, 2);
        assert!(compare(&fixture, &report.text).is_ok());
        let flipped = report.text.replace("local1 == 0", "local1 != 0");
        assert_ne!(flipped, report.text, "the mutation changed the text");
        let error = compare(&fixture, &flipped).expect_err("a flipped condition is a difference");
        assert!(
            error.contains("returned value differs"),
            "the oracle says what differs: {error}"
        );
    }

    #[test]
    fn the_oracle_rejects_a_loop_effect_moved_out_of_the_loop() {
        // The invariant a `while` statement can silently break: the body's effect hoisted out of the
        // loop runs once instead of once per iteration.
        let fixture = while_fixture(0x06);
        let report = recovery_of(&fixture, 3);
        assert!(compare(&fixture, &report.text).is_ok());
        let hoisted = report
            .text
            .replace("        local2.run(0L);\n", "")
            .replace(
                "    Object local2 = null;\n",
                "    Object local2 = null;\n    local2.run(0L);\n",
            );
        assert_ne!(hoisted, report.text, "the mutation changed the text");
        let error = compare(&fixture, &hoisted).expect_err("a hoisted effect is a difference");
        assert!(error.contains("calls differ"), "{error}");
        // And the same invariant stated the other way: dropping the call from the loop entirely is
        // a difference the oracle sees as well.
        let dropped = report.text.replace("        local2.run(0L);\n", "");
        assert_ne!(dropped, report.text, "the mutation changed the text");
        assert!(compare(&fixture, &dropped).is_err());
    }

    #[test]
    fn the_oracle_rejects_a_wrong_return() {
        let fixture = while_fixture(0x06);
        let report = recovery_of(&fixture, 3);
        let wrong = report.text.replace("return 7;", "return 8;");
        assert_ne!(wrong, report.text, "the mutation changed the text");
        let error = compare(&fixture, &wrong).expect_err("a wrong return is a difference");
        assert!(error.contains("returned value differs"), "{error}");
    }

    #[test]
    fn the_oracle_states_the_priority_of_a_crossing_table_and_the_report_carries_it() {
        // The priority is the table's own fact: the first record that covers the throw site. The
        // oracle computes it from the declared table alone, runs the real fixture through the real
        // entry point, and requires the report to state the same record first — while refusing to
        // claim a structure whose handler order it cannot state.
        assert_eq!(
            primary_record(&CROSSING_TABLE, 3),
            Some(0),
            "the first record covers the site"
        );
        assert_eq!(
            primary_record(&CROSSING_TABLE, 4),
            Some(1),
            "only the second one covers BCI 4"
        );
        let class = crossing_class();
        let analysis = analyze(&class);
        let facts = RecoveryFacts::new(MethodFacts::new("method", "()V", 0));
        let mut budget = Budget::new(limits());
        let report = recover(
            &RecoveryRequest::new(analysis.ir(), &facts, crate::pass::JAVA_8),
            &mut budget,
        );
        assert!(report.produced(), "{:?}", report.outcome);
        assert_eq!(
            report.fallbacks,
            vec!["jre_region_crossing_exception_regions"],
            "the crossing table is quoted, not structured: {:?}",
            report.regions
        );
        let message = report
            .regions
            .iter()
            .find_map(|region| region.message.clone())
            .expect("a quoted region states why");
        let primary = primary_record(&CROSSING_TABLE, 3).expect("a record covers the site");
        assert!(
            message.contains(&format!("records {primary} and")),
            "the record a throw reaches first is named first: {message}"
        );
    }

    #[test]
    fn the_oracle_names_no_helper_of_the_layers_it_checks() {
        // The self-guard: the model between the markers must not name anything the recovery layers
        // own. If the oracle ever starts calling the thing it is supposed to be a second opinion
        // about, this test is the one that goes red.
        let source = include_str!("oracle.rs");
        let start = source
            .find("// ------------------------------- oracle model")
            .expect("the model's opening marker");
        let end = source
            .find("// ----------------------------- end oracle model")
            .expect("the model's closing marker");
        let model = &source[start..end];
        for token in [
            "region::",
            "Region",
            "ast::",
            "StmtKind",
            "ExprKind",
            "NormalFlowView",
            "SourceMap",
            "text_of_bci",
            "build::",
            "recover(",
        ] {
            assert!(
                !model.contains(token),
                "the oracle's model names `{token}`, which belongs to the layer it checks"
            );
        }
        // And it is an oracle: the bytecode model, the artifact model and the comparator are all
        // inside the fence.
        for token in ["fn run_bytecode", "fn run_text", "fn compare", "fn parse("] {
            assert!(model.contains(token), "the model holds `{token}`");
        }
    }
}
