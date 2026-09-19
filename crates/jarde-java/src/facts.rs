//! The facts the recovery layer renders from but does not own.
//!
//! # Why this input exists at all
//!
//! The 1.1 IR handoff publishes the method's *structure*: canonical blocks and their covered BCIs,
//! edges, frames, SSA values with their definitions and uses, effects. It publishes no **symbolic
//! vocabulary** — a constant-pool reference's owner/name/descriptor, the value a `ldc`/`iconst`
//! pushes, which local a `*load`/`*store` names, whether a branch jumps when its value is zero.
//! Those are decode facts the layer below read out of the class file, and they are not in the
//! payload (P3 1.1's own record notes the payload holds no CP index; the effect facts carry the
//! opcode byte and the BCIs, not the operands).
//!
//! A presentation cannot be written without them: `local1 = local2 + local3;` needs to know which
//! slot the instruction read, and `if (local1 != 0)` needs to know which way round an `ifeq` is.
//! So they enter through this module: **facts about the bytecode, supplied by the caller that read
//! them**, exactly the way the debug names and the method's identity do. The recovery layer decides
//! syntax; it does not decode.
//!
//! # The boundary this keeps
//!
//! [`Operation`] is deliberately *not* a statement. It says "this BCI reads local 1", never "this
//! BCI is an assignment to `count`" — the spelling, the statement shape, the declaration, the
//! negation of a branch's polarity (the emitter writes the fall-through condition, which is the
//! negation of the jump sense) and the origin of every node are all decided above this seam. An
//! operation the subset does not model — a field access, a `switch`, an array operation, a
//! conversion — is [`Operation::Other`] and makes the statement it belongs to unrenderable, which
//! is a declared fallback with a diagnostic and never an invented expression.

use std::collections::BTreeMap;

/// What a class file says about the method whose body is being presented.
///
/// Identity only: the name and descriptor are evidence the presentation quotes, and the parameter
/// count is what tells an ordinal name whether it belongs to a parameter or to a local.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodFacts {
    name: String,
    descriptor: String,
    parameters: u16,
}

impl MethodFacts {
    /// One method's identity: its name, its descriptor and its number of parameter slots.
    pub fn new(name: impl Into<String>, descriptor: impl Into<String>, parameters: u16) -> Self {
        Self {
            name: name.into(),
            descriptor: descriptor.into(),
            parameters,
        }
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
/// [`crate::region`] for the rule and `crate::build` for where it is applied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompareOp {
    /// One value: transfer when it is zero (`ifeq`, `ifnull`).
    JumpIfZero,
    /// One value: transfer when it is non-zero (`ifne`, `ifnonnull`).
    JumpIfNotZero,
    /// Two values: transfer when they are equal (`if_icmpeq`, `if_acmpeq`).
    JumpIfSame,
    /// Two values: transfer when they differ (`if_icmpne`, `if_acmpne`).
    JumpIfDifferent,
}

impl CompareOp {
    /// Whether the instruction reads two values rather than one.
    pub fn reads_two(self) -> bool {
        matches!(self, Self::JumpIfSame | Self::JumpIfDifferent)
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
    /// Transfers on a condition built from the values it reads.
    Comparison { op: CompareOp },
    /// A plain transfer the *structure* already stands for — a `goto` inside a recovered region.
    ///
    /// It is neither a value nor an effect on the program's state, so it becomes no statement; and it
    /// is not [`Self::Other`] either, because the structure layer did model what it does (it is an
    /// edge of the region it belongs to), which is exactly the difference between "this layer does
    /// not model the operation" and "the operation leaves no statement behind".
    Transfer,
    /// Invokes the target it names, with the receiver and arguments it reads.
    Invoke(CallTarget),
    /// Leaves the method.
    Return,
    /// Anything else: legal to read, not part of the provable subset.
    ///
    /// The variant exists so that "this layer has not modelled the operation yet" is a *stated*
    /// input rather than an unstated assumption. A statement built on one becomes a fallback with a
    /// diagnostic, which is what keeps a `switch`, a field access or an array store from being
    /// printed as a guess before 2.x/3.x land.
    Other,
}

/// Every fact the recovery run needs beyond the IR payload: the method, its debug names, and the
/// decoded operations of its body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryFacts {
    method: MethodFacts,
    debug_locals: Vec<Option<String>>,
    operations: BTreeMap<u32, Operation>,
}

impl RecoveryFacts {
    /// The facts of one method, with no debug names and no operations yet.
    pub fn new(method: MethodFacts) -> Self {
        Self {
            method,
            debug_locals: Vec::new(),
            operations: BTreeMap::new(),
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

    /// The same facts with one operation decoded for one bytecode index.
    pub fn with_operation(mut self, bci: u32, operation: Operation) -> Self {
        self.operations.insert(bci, operation);
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

    /// What the instruction at one bytecode index does, when the caller decoded it.
    pub fn operation(&self, bci: u32) -> Option<&Operation> {
        self.operations.get(&bci)
    }

    /// Every decoded operation, in BCI order.
    pub fn operations(&self) -> impl Iterator<Item = (&u32, &Operation)> {
        self.operations.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facts_are_read_back_where_they_were_written() {
        let facts = RecoveryFacts::new(MethodFacts::new("add", "(II)I", 2))
            .with_debug_locals(vec![Some("left".into()), Some("right".into())])
            .with_operation(0, Operation::Load { slot: 0 })
            .with_operation(
                3,
                Operation::Invoke(CallTarget::new(
                    InvokeKind::Static,
                    "java/lang/Math",
                    "max",
                    "(II)I",
                )),
            );
        assert_eq!(facts.method().name(), "add");
        assert_eq!(facts.operation(0), Some(&Operation::Load { slot: 0 }));
        assert_eq!(
            facts.operation(1),
            None,
            "an undecoded BCI has no operation"
        );
        assert_eq!(facts.debug_locals().len(), 2);
        assert_eq!(
            facts.operations().map(|(bci, _)| *bci).collect::<Vec<_>>(),
            vec![0, 3],
            "BCI order"
        );
        assert!(CompareOp::JumpIfSame.reads_two());
        assert!(!CompareOp::JumpIfZero.reads_two());
        match facts.operation(3) {
            Some(Operation::Invoke(target)) => {
                assert_eq!(target.descriptor(), "(II)I");
                assert_eq!(target.kind(), InvokeKind::Static);
            }
            other => panic!("expected the invocation, got {other:?}"),
        }
    }
}
