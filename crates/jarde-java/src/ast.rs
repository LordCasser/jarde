//! The minimal Java statement and expression AST of the provable subset (P3 1.2 decision 1), with
//! one [`OriginSet`] per node.
//!
//! # What the subset is
//!
//! Straight-line bodies and `if`/`else`, in Java statement form: local declarations, assignments,
//! call statements, `return`, `if`/`else`, and — where the evidence runs out — an explicit
//! [`StmtKind::Fallback`] that quotes the bytecode it could not present. Expressions are locals,
//! integer/long/String/null constants, calls (with a receiver or on a type), and the binary
//! operators the arithmetic and comparison facts name.
//!
//! # What it deliberately is not
//!
//! Not a general Java AST. There is no lambda, no `switch`, no `try`, no field access node, no
//! generics, no annotations, no modifiers, no method envelope — the 2.x patterns add those one at a
//! time, each with the pattern preconditions that make it provable, and P3 decision 2 keeps this
//! slice to the subset whose recovery this layer can *prove* from the IR. A node the subset cannot
//! build is a fallback, never an invented statement.
//!
//! # Why the origin lives on the node and not beside the tree
//!
//! The emitter writes text and segments in one pass (P3 decision 3), so each node has to carry what
//! its own text came from. Keeping the anchors on the node also means a consumer never has to pair
//! a tree walk with a parallel table by position — the association cannot go out of step.

use crate::source_map::OriginSet;

/// A Java type, as this subset writes it: the primitives whose frames the IR can state, plus a
/// reference type named by the resolution the frames carry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    /// `boolean`, `byte`, `char` and `short`: a **descriptor** states which of the int-shaped
    /// primitives a position has (a `MethodType` bootstrap argument says `Z` outright), while the
    /// *frames* cannot — a `boolean` and an `int` share one slot and one value shape — so
    /// [`crate::build`]'s frame reading never produces these four and only the descriptor reader of
    /// a lambda's SAM ([`crate::lambda`]) does. That is the difference between a fact a descriptor
    /// states and a guess this layer would be making.
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    /// A reference type in source form (`java.lang.String`), or `Object` when the frames state a
    /// reference without naming it.
    Reference(String),
}

impl Type {
    /// The Java spelling of this type.
    pub fn spell(&self) -> &str {
        match self {
            Self::Boolean => "boolean",
            Self::Byte => "byte",
            Self::Char => "char",
            Self::Short => "short",
            Self::Int => "int",
            Self::Long => "long",
            Self::Float => "float",
            Self::Double => "double",
            Self::Reference(name) => name,
        }
    }
}

/// A binary operator this subset can write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    /// `==` as it appears in a condition.
    Equal,
    /// `!=` as it appears in a condition.
    NotEqual,
    /// `<` as it appears in a condition.
    Less,
    /// `<=` as it appears in a condition.
    LessOrEqual,
    /// `>` as it appears in a condition.
    Greater,
    /// `>=` as it appears in a condition.
    GreaterOrEqual,
}

impl BinaryOp {
    /// The Java spelling of this operator.
    pub fn spell(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Remainder => "%",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Less => "<",
            Self::LessOrEqual => "<=",
            Self::Greater => ">",
            Self::GreaterOrEqual => ">=",
        }
    }
}

/// What one expression is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExprKind {
    /// A local or parameter name, already decided by [`crate::names`].
    Local(String),
    /// An `int`-shaped literal.
    Integer(i64),
    /// A `long` literal, written with its `L` suffix.
    Long(i64),
    /// A string literal; the raw value is held here and escaped by the emitter, so no expression
    /// carries an escape decision that the writer could then disagree with.
    Str(String),
    /// `null`.
    Null,
    /// A type name used as an expression, as a static call's receiver: `java.lang.Math`.
    Path(String),
    /// A call: an optional receiver expression, a member name, and the argument expressions.
    Call {
        receiver: Option<Box<Expr>>,
        name: String,
        args: Vec<Expr>,
    },
    /// `new Type(args…)` — the body of a lambda whose implementation handle is a constructor.
    New { ty: String, args: Vec<Expr> },
    /// `(params) -> body` — a lambda expression, with the parameters the SAM states.
    ///
    /// The parameters are written with their types ([`LambdaParam`]) rather than left to inference:
    /// the target type may not be declared at all in a recovered body (the store's type is only
    /// known when the frames state one), and a typed parameter list states exactly what the SAM's
    /// descriptor said — the same evidence the record reads back.
    Lambda {
        params: Vec<LambdaParam>,
        body: Box<Expr>,
    },
    /// `Qualifier::name` — a method reference, with the qualifier a type or an expression.
    MethodReference { qualifier: Box<Expr>, name: String },
    /// A binary operation over two expressions.
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

/// One parameter of a lambda: the type its SAM states and the name this layer gave it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LambdaParam {
    /// The parameter's type, as the implementation handle's own descriptor states it.
    pub ty: Type,
    /// The name the presentation gave it ([`crate::names::NameTable::free_name`]): the class file
    /// does not name a lambda's parameters, so the name is derived and guaranteed not to collide
    /// with any name the body's own locals carry.
    pub name: String,
}

/// One expression and the anchors behind its text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expr {
    /// What the expression is.
    pub kind: ExprKind,
    /// Where its text comes from.
    pub origin: OriginSet,
}

impl Expr {
    /// One expression from its shape and its anchors.
    pub fn new(kind: ExprKind, origin: OriginSet) -> Self {
        Self { kind, origin }
    }

    /// One expression anchored directly at one bytecode index.
    pub fn direct(kind: ExprKind, bci: u32) -> Self {
        Self::new(kind, OriginSet::new(crate::source_map::Origin::direct(bci)))
    }

    /// The same expression, presenting a further anchor.
    pub fn derived_from(mut self, bci: u32) -> Self {
        self.origin = self
            .origin
            .clone()
            .plus_derived(crate::source_map::Origin::derived(bci));
        self
    }
}

/// What one statement is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StmtKind {
    /// `int local1;` or `int local1 = <expr>;` — a slot's declared type, stated at its first write.
    ///
    /// The initialiser is optional because a slot's first write may not carry a value this layer can
    /// render; the declaration is still written, and the assignment that follows states the value.
    Declare {
        ty: Type,
        name: String,
        value: Option<Expr>,
    },
    /// `local1 = <expr>;`
    Assign { name: String, value: Expr },
    /// `<expr>;` — a call whose result is not used.
    Expr(Expr),
    /// `return;` or `return <expr>;`
    Return { value: Option<Expr> },
    /// `if (<cond>) { … } else { … }`, with an empty `else_body` when the source had none.
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },
    /// `while (<cond>) { … }` — the test runs before every iteration, the body only when it holds.
    While { cond: Expr, body: Vec<Stmt> },
    /// `do { … } while (<cond>);` — the body runs once before the test is read.
    DoWhile { cond: Expr, body: Vec<Stmt> },
    /// `switch (<value>) { … }`, with one arm per distinct target of the decoded `switch`.
    Switch { value: Expr, arms: Vec<SwitchArm> },
    /// Text this slice could not present as Java, quoting what it could not present.
    ///
    /// This is the declared fallback of the recoverable-subset boundary: the bytecode is quoted
    /// (the emitter writes `// @bytecode 44 45 46` and the reason) so the artifact still says what
    /// the region was, and the report states the weaker representation and quality that go with it.
    /// It is never an empty body: a region that cannot be presented produces this node or the run
    /// stops and produces no text at all.
    Fallback { reason: String, bcis: Vec<u32> },
}

/// One arm of a `switch` statement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwitchArm {
    /// The labels this arm is reached by. Empty means the arm is the no-match case alone.
    pub keys: Vec<i64>,
    /// Whether the no-match case reaches this arm too (a `default:` label beside the keys).
    pub default: bool,
    /// The statements of the arm, each of which runs at most once per execution of the switch.
    pub body: Vec<Stmt>,
}

/// One statement and the anchors behind its text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stmt {
    /// What the statement is.
    pub kind: StmtKind,
    /// Where its text comes from.
    pub origin: OriginSet,
}

impl Stmt {
    /// One statement from its shape and its anchors.
    pub fn new(kind: StmtKind, origin: OriginSet) -> Self {
        Self { kind, origin }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_map::Origin;

    #[test]
    fn a_node_carries_its_own_anchors() {
        let call = Expr::new(
            ExprKind::Integer(1),
            OriginSet::derived_from(Origin::direct(4), Origin::derived(9)),
        );
        assert_eq!(call.origin.primary().bci(), 4);
        assert_eq!(call.origin.derived().len(), 1);
        let derived = call.derived_from(12);
        assert_eq!(derived.origin.derived().len(), 2);
        assert_eq!(derived.origin.bcis().len(), 3);
    }

    #[test]
    fn the_subset_spells_the_operators_and_types_it_claims() {
        assert_eq!(BinaryOp::Remainder.spell(), "%");
        assert_eq!(BinaryOp::NotEqual.spell(), "!=");
        assert_eq!(
            Type::Reference("java.lang.String".into()).spell(),
            "java.lang.String"
        );
        assert_eq!(Type::Long.spell(), "long");
    }
}
