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

use serde::Serialize;

use crate::source_map::OriginSet;

/// A Java type, as this subset writes it: the primitives whose frames the IR can state, plus a
/// reference type named by the resolution the frames carry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    /// `boolean`, `byte`, `char` and `short`: a **descriptor** states which of the int-shaped
    /// primitives a position has (a `MethodType` bootstrap argument says `Z` outright), while the
    /// *frames* cannot — a `boolean` and an `int` share one slot and one value shape — so the frame
    /// reading never produces these four on its own: [`crate::build`] produces [`Self::Boolean`]
    /// from a descriptor (a `Z` return, a `Z` parameter or callee, a claimed field, a lambda's SAM)
    /// and, transitively, from a local this body already declared `boolean`, and [`crate::lambda`]
    /// spells a SAM parameter from the same kind of fact. That is the difference between a fact a
    /// descriptor states and a guess this layer would be making.
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
    /// `true`/`false` — a boolean constant.
    ///
    /// This node exists for the same reason [`ExprKind::Not`] does, read from the other side: a
    /// `boolean` parameter and an `int` one are one slot shape and the bytecode pushes the
    /// `int`-shaped `1`/`0` for `true`/`false`, so the *call site* cannot state which of the two it
    /// passes and the **callee's own descriptor** is the only evidence that can. [`crate::build`]
    /// writes this node where that descriptor declares the parameter `boolean` and the argument is a
    /// literal `0`/`1`; every other argument keeps the literal it was rendered as (P3-R5's argument
    /// side).
    Boolean(bool),
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
    /// `receiver.name` — one field of an instance, or `Type.name` for a static field with the type
    /// as the receiver — written where a rule proved which member the instruction names.
    ///
    /// Two shapes reach this node: a call site whose callee's body was verified as the pure
    /// forwarding of one field access ([`crate::accessor`], where the receiver is the value the call
    /// site passed), and a `getfield`/`getstatic` of the presented body itself where the `field@1`
    /// rule proved the member it reads ([`crate::field`], where the receiver is the value the
    /// instruction read). A field access **no** rule claimed never becomes this node: it is quoted
    /// as bytecode.
    Field { receiver: Box<Expr>, name: String },
    /// `array[index]` — one element of an array, written where the value is consumed.
    ///
    /// This node is written for one shape: the `int`-shaped dispatch table a compiler's `switch`
    /// over an enum reads its case index out of ([`crate::enumswitch`]). An array read no rule
    /// claimed stays quoted.
    Index { array: Box<Expr>, index: Box<Expr> },
    /// A binary operation over two expressions.
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// `!value` — the negation of a **boolean** value.
    ///
    /// This node exists for one fact the frames cannot state: a `boolean` parameter and an `int`
    /// parameter share one slot shape, so `ifeq` on slot 1 is `b != 0` for an `int` and it is `!b`
    /// for a `boolean` — and only the method's own descriptor says which. [`crate::facts`] reads that
    /// descriptor, and a zero test on a `boolean`-typed slot is written with this node rather than as
    /// a comparison the method's signature would refuse to compile (P3-R5).
    Not { value: Box<Expr> },
}

/// Which constructor one instance initializer calls first.
///
/// The two are different programs, so this is a verdict of a rule and never a convention: a call on
/// the frames' `UninitializedThis` whose class the caller stated to be the declaring class is
/// [`Self::This`], and one whose class is not is [`Self::Super`] — JVMS 4.9.2 leaves an instance
/// initializer no third option, and the run that cannot state the declaring class refuses to spell
/// either.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructorTarget {
    /// Another constructor of the same class: `this(…)`.
    This,
    /// The direct superclass's constructor: `super(…)`.
    Super,
}

impl ConstructorTarget {
    /// The Java keyword this target is written with.
    pub fn spell(self) -> &'static str {
        match self {
            Self::This => "this",
            Self::Super => "super",
        }
    }
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
    /// `receiver.name = value;` — the write a synthetic accessor's call performed, or one a
    /// `putfield`/`putstatic` of the presented body performed where `field@1` proved the member it
    /// writes.
    ///
    /// A write accessor returns nothing, so the call site that used to spell it is a statement:
    /// the receiver is the first argument the site passed, and the value is the second. A `putfield`
    /// reaches the same node with the value and the receiver the instruction itself read, and a
    /// `putstatic` with the owner type as the receiver.
    FieldAssign {
        receiver: Expr,
        name: String,
        value: Expr,
    },
    /// `super(args);` or `this(args);` — the constructor call an instance initializer starts with.
    ///
    /// Which of the two it is, is read and not guessed ([`crate::init`]): the call's receiver is the
    /// frames' `UninitializedThis`, which only a constructor's own `this` before its constructor
    /// call can be, and what decides between the two spellings is whether the class the call names
    /// is the class that declares the body (JVMS 4.9.2 lets an instance initializer call exactly its
    /// own class's constructor or its superclass's). Writing `super` for a `this` call — or the
    /// reverse — would run another constructor, so an unproven prologue is quoted instead.
    ConstructorCall {
        target: ConstructorTarget,
        args: Vec<Expr>,
    },
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
    /// `try (T n = expr; …) { … }` — the guarded statement the `twr@1` rule proved.
    ///
    /// The resources are written in **declaration** order, which is the order the rule read their
    /// initialisations in, and the compiler closes them in the reverse order — the order the bytecode
    /// really closed them in, because that is the order the rule matched the normal path's close
    /// chain against before it may write this node ([`crate::guard`]).
    Try {
        resources: Vec<ResourceDecl>,
        body: Vec<Stmt>,
    },
    /// `synchronized (<lock>) { … }` — the guarded statement the `monitor@1` rule proved, with every
    /// path out of the body leaving the monitor it entered.
    Synchronized { lock: Expr, body: Vec<Stmt> },
    /// Text this slice could not present as Java, quoting what it could not present.
    ///
    /// This is the declared fallback of the recoverable-subset boundary: the bytecode is quoted
    /// (the emitter writes `// @bytecode 44 45 46` and the reason) so the artifact still says what
    /// the region was, and the report states the weaker representation and quality that go with it.
    /// It is never an empty body: a region that cannot be presented produces this node or the run
    /// stops and produces no text at all.
    Fallback { reason: String, bcis: Vec<u32> },
}

/// One resource of a `try` header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceDecl {
    /// The resource's declared type, as the frames state the value's own type. A value the frames
    /// name as no reference type at all is refused by the rule that would write the header, never
    /// spelled `Object`.
    pub ty: Type,
    /// The name the presentation gave the slot the resource lives in.
    pub name: String,
    /// The initialisation expression: the value the header's declaration is filled with. Its text is
    /// the one the initialisation's own instructions produce, written where the statement runs them.
    pub value: Expr,
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
