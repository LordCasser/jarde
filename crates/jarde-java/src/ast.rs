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
    LeftShift,
    RightShift,
    UnsignedRightShift,
    /// `&` as an integral or eager boolean operation.
    BitwiseAnd,
    /// `^` as an integral or eager boolean operation.
    BitwiseXor,
    /// `|` as an integral or eager boolean operation.
    BitwiseOr,
    /// `&&` — a proved, short-circuiting boolean conjunction.
    LogicalAnd,
    /// `||` — a proved, short-circuiting boolean disjunction.
    LogicalOr,
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
            Self::LeftShift => "<<",
            Self::RightShift => ">>",
            Self::UnsignedRightShift => ">>>",
            Self::BitwiseAnd => "&",
            Self::BitwiseXor => "^",
            Self::BitwiseOr => "|",
            Self::LogicalAnd => "&&",
            Self::LogicalOr => "||",
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
    /// A `float` literal, written from the **raw bits** the class file stated.
    ///
    /// The `u32` is the exact bit pattern of the constant (JLS 3.10.2's hexadecimal spelling
    /// round-trips it bit for bit), so no host `f32` normalization ever stands between the class
    /// file's value and the text. The leaf carries **finite** bits only: infinities and the one
    /// admitted NaN are presented where the constant is read, as the proved constant divisions of
    /// these leaves, and a NaN pattern the change cannot present exactly is a refusal rather than
    /// a leaf.
    Float(u32),
    /// A `double` literal, by the same raw-bits rule as [`ExprKind::Float`].
    Double(u64),
    /// A string literal; the raw value is held here and escaped by the emitter, so no expression
    /// carries an escape decision that the writer could then disagree with.
    Str(String),
    /// `null`.
    Null,
    /// `T.class` — a Java class literal whose type name was validated from a `CONSTANT_Class` pool
    /// entry reached by `ldc` or `ldc_w`.
    ClassLiteral { ty: String },
    /// A JVM reference type test, spelled with Java's relational precedence.
    InstanceOf { value: Box<Expr>, ty: String },
    /// A type name used as an expression, as a static call's receiver: `java.lang.Math`.
    Path(String),
    /// `super` or `Type.super`, used only as the receiver of a proven `invokespecial` call.
    Super { qualifier: Option<String> },
    /// A call: an optional receiver expression, a member name, and the argument expressions.
    Call {
        receiver: Option<Box<Expr>>,
        name: String,
        args: Vec<Expr>,
    },
    /// `new Type(args…)`, or a proved member creation `receiver.new Inner(args…)`.
    ///
    /// `ty` always retains the complete semantic allocation type. The optional simple member
    /// name is only the source spelling for a proved member construction; it is not used to infer
    /// the constructed type from a binary name.
    New {
        ty: String,
        qualifier: Option<Box<Expr>>,
        member_name: Option<String>,
        /// Only a proved generic member target may request `receiver.new Inner<>(args…)`.
        diamond: bool,
        args: Vec<Expr>,
    },
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
    /// Two shapes reach this node: the `int`-shaped dispatch table a compiler's `switch` over an
    /// enum reads its case index out of ([`crate::enumswitch`]), and the ordinary read of one
    /// element (P3 2b) — `a[i]` for a read of an `int[]`, a `char[]`, a `boolean[]` or an array of
    /// references, whose element type is the array's own. An array read this layer cannot present —
    /// one whose array or index is bytecode it cannot write — stays quoted, like every other value
    /// whose producers refuse.
    ///
    /// The element type is not on the node: the text is the same `array[index]` whatever the
    /// element is, and what the element *is* is stated by the type the node presents
    /// ([`Expr::presented`]), which [`crate::build`] reads from the array's own type.
    Index { array: Box<Expr>, index: Box<Expr> },
    /// `target++` — the old value of one already-proved writable `int` field or `int[]` element,
    /// after the target has been incremented.
    ///
    /// The target remains an expression so its receiver, array and index retain their own grouping
    /// and origins. Recovery builds this node only after proving the bytecode's read, write and old
    /// value return refer to the same writable target.
    PostIncrement { target: Box<Expr> },
    /// `array.length` — the length of an array, written where the value is consumed.
    ///
    /// It is a node of its own and never a field access (P3 2b): `array.length` is an operator of
    /// the array type and not a member this layer would have to prove, and the `field@1` rule's
    /// proofs must not be asked about it.
    ArrayLength { array: Box<Expr> },
    /// `new T[n]`, `new T[n][m][]` for a partial multi-dimensional creation, or a proved
    /// `new T[]{…}` — one array allocation.
    ///
    /// `element` is the **element** type the creation states, `lengths` is one length per
    /// dimension the instruction allocates, `initializers` is present only when a same-block
    /// store chain was proved complete at every included array level, and
    /// `total_dimensions` is the complete array rank.
    /// The type of the expression is the full array (`int[][]` for one allocated prefix of rank
    /// two), which [`Expr::presented`] carries; the element type is spelled as a Java type
    /// ([`Type::spell`]).
    ///
    /// A creation whose element type no descriptor, `atype` code or pool class states is a
    /// [`StmtKind::Fallback`]. An unproved initializer remains its existing array statements or a
    /// complete fallback; it never becomes this node with only part of the chain attached.
    NewArray {
        element: Type,
        lengths: Vec<Expr>,
        initializers: Option<Vec<Expr>>,
        total_dimensions: u8,
    },
    /// A binary operation over two expressions.
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// `test ? when_true : when_false` — a stack value whose two inputs and consumer were proved
    /// to be the straight arms of one `if`. Its type is stated only when the two arms have the
    /// same primitive/reference type, or one arm is `null` and the other states a reference type.
    Conditional {
        test: Box<Expr>,
        when_true: Box<Expr>,
        when_false: Box<Expr>,
    },
    /// A verified concatenation chain: one [`ConcatPart`] per `append`, in the order the chain
    /// calls them (`concat@1`).
    ///
    /// This node exists because a chain's presentation is **not** a nesting of `Binary` nodes: the
    /// parts are a sequence, and the conversion each `append` performs belongs to the part it read
    /// (see [`ConcatPart`]). Two consequences are the reason the node is shaped this way:
    ///
    /// * adding one part grows a `Vec` and nothing else — the constructor, the printer, `Clone` and
    ///   `Drop` iterate the sequence, so a chain of any length costs what the chain of that length
    ///   costs and no stack depth per part. A left-deep `Binary` chain has the length *itself* as
    ///   the depth of all four paths, which is what the report's deep-chain finding measured
    ///   (`new StringBuilder().append(s)` repeated 1536/2048 times aborted the process in the
    ///   printer);
    /// * a text printed by concatenating `+` and its operands cannot distinguish `append(a).append(b)`
    ///   from `append(a + b)`: the former is `"" + a + b` (two conversions) and the latter is
    ///   `"" + (a + b)` (one addition, then one conversion), and the parts keep that distinction
    ///   because each part's own expression keeps its own grouping.
    ///
    /// The head of the text is the chain's first `+`, and it is a **string** concatenation: when the
    /// first part is not already a `String`, the printer writes the empty string javac lowers
    /// `"" + a` with in front of it (see [`ConcatPart::is_a_string`]), so the first part's
    /// conversion happens where the chain converts it.
    Concat { parts: Vec<ConcatPart> },
    /// `(int) arg0` — one conversion a **conversion instruction** in the body performed, written by
    /// the text.
    ///
    /// This node exists because such a conversion is a **build-time decision** and never a permission
    /// the printer takes from the position it happens to write into: an `i2l` or an `i2b` is an
    /// instruction of the body, and the value it produced is presented as the narrow type while the
    /// position that reads it may require another one — a text that leaves the conversion to the
    /// position would state a program the bytecode does not have where the two differ.
    ///
    /// What this node is deliberately **not** is a widening a position makes for itself (P3 2c.29). A
    /// `char`, a `byte` and a `short` share one slot shape with an `int`, so a compiler writes no
    /// instruction when an assignment or `return` position widens one of them, and the value's own
    /// text already says what it is. Invocation arguments are different: their descriptor selects an
    /// overload, so the build may add this node to preserve that selection. The build compares the
    /// type the expression presents as ([`Expr::presented`]) with the type the position requires (the
    /// `append`'s parameter descriptor, the callee's parameter, the member's return type, the written
    /// variable's declaration, the field's descriptor) and writes a cast where that consuming position
    /// needs one.
    ///
    /// The printer writes exactly this node and nothing else: a cast's text is its own type and the
    /// value it converts, so a position can no longer decide the value's type by writing the value
    /// in a context of its own choosing.
    Cast { ty: Type, value: Box<Expr> },
    /// `!value` — the negation of a **boolean** value.
    ///
    /// This node exists for one fact the frames cannot state: a `boolean` parameter and an `int`
    /// parameter share one slot shape, so `ifeq` on slot 1 is `b != 0` for an `int` and it is `!b`
    /// for a `boolean` — and only the method's own descriptor says which. [`crate::facts`] reads that
    /// descriptor, and a zero test on a `boolean`-typed slot is written with this node rather than as
    /// a comparison the method's signature would refuse to compile (P3-R5).
    Not { value: Box<Expr> },
    /// `-value` — the numeric negation performed by `ineg`, `lneg`, `fneg` or `dneg`.
    ///
    /// The operand remains a child so nested negations retain their grouping and source anchors.
    /// Its type follows Java's unary numeric promotion: byte, short and char become int, while
    /// int, long, float and double keep the type the opcode produces.
    Neg { value: Box<Expr> },
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
    /// The parameter's type, as the erased SAM descriptor states it. Checks and implementation
    /// conversions belong inside the lambda body, after the call supplies this parameter.
    pub ty: Type,
    /// The name the presentation gave it ([`crate::names::NameTable::free_name`]): the class file
    /// does not name a lambda's parameters, so the name is derived and guaranteed not to collide
    /// with any name the body's own locals carry.
    pub name: String,
}

/// One `append` of a presented concatenation chain: the value it read, and the parameter type its
/// own pool reference declares.
///
/// The parameter type is the part's **target position**, and it is what makes the part's text the
/// text of *that* `append` rather than of a `+` on the value's own type: `"" + value` writes what
/// `append` writes only while the descriptor says the two conversions are the same one
/// ([`crate::concat`] accepts exactly those overloads), and `boolean` is the one conversion whose
/// text the value does not already carry — a `boolean` is pushed as the `int`-shaped `0`/`1`
/// (`append(true)`'s `iconst_1`), so `true`/`false` is a spelling only the descriptor's `Z` states.
/// [`crate::build`] applies that conversion where the descriptor and the value's own evidence meet,
/// and the printer consults the parameter for the one text decision that is the chain's own: whether
/// the first `+` is already a string concatenation ([`Self::is_a_string`]).
///
/// The part's anchors are on `value`: the value's own producers as its direct anchors, and the
/// `append` that converted it as a presented one — one part per `append`, in the bytecode's order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcatPart {
    /// The parameter type the `append` instruction's own pool reference states, as the chain's own
    /// rule read it out of the pool (`crate::concat::Chain::appends`).
    pub parameter: Type,
    /// The value it read, written where the chain converts it (`crate::build::concat_expr`).
    pub value: Expr,
}

impl ConcatPart {
    /// One part from its `append`'s parameter type and the value it read.
    pub fn new(parameter: Type, value: Expr) -> Self {
        Self { parameter, value }
    }

    /// Whether this part is **already** a `String`, so that a `+` starting at it is a string
    /// concatenation without any decoration.
    ///
    /// This is the whole string-context rule: the text of a chain whose first part is a `String`
    /// gains nothing, and one whose first part needs `String.valueOf` starts from the empty string
    /// instead of adding that part as a number. An `append` whose parameter is `Object` while the
    /// value it read is proven to be a `String` is *not* a string part: its descriptor is `Object`,
    /// and the empty string it starts from is the identity the conversion it stands for has.
    pub fn is_a_string(&self) -> bool {
        matches!(&self.parameter, Type::Reference(name) if name == "java.lang.String")
    }
}

/// One expression and the anchors behind its text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expr {
    /// What the expression is.
    pub kind: ExprKind,
    /// Where its text comes from.
    pub origin: OriginSet,
    /// The type this expression's **text** is presented as, when this layer's own evidence states
    /// one.
    ///
    /// This is the value the consuming positions read: a position whose **text** performs a
    /// conversion (`+`'s string part, P3 2c.29's `Widening::Text`) states it, while assignment and
    /// return positions Java converts implicitly write the value as it is. Invocation arguments use
    /// the callee descriptor and may add an explicit [`ExprKind::Cast`] to preserve overload
    /// selection. A real conversion instruction is also a [`ExprKind::Cast`] the value's own layer
    /// writes. `None` is "this
    /// layer states no type", never "no type": a `null`, a lambda and a method reference each state
    /// none (a lambda's target type is not declared anywhere in a recovered body), an arithmetic
    /// whose operands state none states none, and a position that requires a type converts nothing
    /// whose own type it cannot state.
    ///
    /// Every type here is a **descriptor** or **declaration** fact of the run and never a guess from
    /// the value's shape: the frames state one slot shape for the four int-sized primitives, so
    /// `char`, `byte` and `short` reach this field from the member's own parameter descriptor and
    /// from nothing else.
    pub presented: Option<Type>,
}

impl Expr {
    /// One expression from its shape and its anchors, presenting the type its own shape states.
    pub fn new(kind: ExprKind, origin: OriginSet) -> Self {
        let presented = presented_of(&kind);
        Self {
            kind,
            origin,
            presented,
        }
    }

    /// One expression anchored directly at one bytecode index.
    pub fn direct(kind: ExprKind, bci: u32) -> Self {
        Self::new(kind, OriginSet::new(crate::source_map::Origin::direct(bci)))
    }

    /// The same expression, presenting the type a fact of its **position** states: the descriptor a
    /// callee declares, the type a call's result has, the declaration a local's variable was given,
    /// the descriptor a claimed field access names.
    ///
    /// [`Self::new`] states what an expression's own shape states; this states what only the run's
    /// tables hold, and it is called at the moment the node is built — where that fact is at hand —
    /// so no consumer has to look a name or a BCI up again to learn what the text is.
    pub fn presenting(mut self, ty: Type) -> Self {
        self.presented = Some(ty);
        self
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

/// The type one expression states **by its own shape**, with no other fact consulted.
///
/// Three shapes state nothing and are the honest `None`s of [`Expr::presented`]: `null`, which has no
/// type; a lambda and a method reference, whose target type a recovered body does not declare (it is
/// the class the site is assigned to, which is a fact of the *assignment* and not of the site); and a
/// type name used as a call's receiver, which is not a value at all.
///
/// The three shapes that state their type from a fact outside their own text — a local (the
/// declaration its variable was given), a call (the callee's return descriptor) and a field read
/// (the field's descriptor) — are `None` here and are given theirs where the node is built
/// ([`Expr::presenting`]).
fn presented_of(kind: &ExprKind) -> Option<Type> {
    match kind {
        ExprKind::Integer(_) => Some(Type::Int),
        ExprKind::Long(_) => Some(Type::Long),
        // The leaf's own bits state its type: a float literal is a float, a double literal a
        // double, and the presentation never widens one into the other.
        ExprKind::Float(_) => Some(Type::Float),
        ExprKind::Double(_) => Some(Type::Double),
        ExprKind::Boolean(_) => Some(Type::Boolean),
        ExprKind::Str(_) => Some(Type::Reference("java.lang.String".to_string())),
        ExprKind::ClassLiteral { .. } => Some(Type::Reference("java.lang.Class".to_string())),
        ExprKind::New { ty, .. } => Some(Type::Reference(ty.clone())),
        // An array's length is an `int` (JLS 10.7), whatever the array's element is.
        ExprKind::ArrayLength { .. } => Some(Type::Int),
        // Java's postfix increment expression has the type of its variable; this recovery node is
        // built only for the proved `int` field and array-element shapes.
        ExprKind::PostIncrement { target } => target.presented.clone(),
        // A creation's own shape states the array it builds: the element type and one dimension per
        // length. A node with no length at all states no array — it builds nothing — and presents
        // none rather than the bare element.
        ExprKind::NewArray {
            element,
            lengths,
            initializers,
            total_dimensions,
        } => {
            if initializers.is_some() {
                (lengths.is_empty() && *total_dimensions > 0).then(|| {
                    Type::Reference(format!(
                        "{}{}",
                        element.spell(),
                        "[]".repeat(usize::from(*total_dimensions))
                    ))
                })
            } else {
                (!lengths.is_empty() && usize::from(*total_dimensions) >= lengths.len()).then(
                    || {
                        Type::Reference(format!(
                            "{}{}",
                            element.spell(),
                            "[]".repeat(usize::from(*total_dimensions))
                        ))
                    },
                )
            }
        }
        ExprKind::Cast { ty, .. } => Some(ty.clone()),
        ExprKind::Concat { .. } => Some(Type::Reference("java.lang.String".to_string())),
        ExprKind::Not { .. } => Some(Type::Boolean),
        ExprKind::InstanceOf { .. } => Some(Type::Boolean),
        ExprKind::Neg { value } => promotion_rank(value.presented.as_ref()?).map(promoted),
        ExprKind::Binary { op, left, right } => binary_type(*op, left, right),
        ExprKind::Conditional {
            when_true,
            when_false,
            ..
        } => match (when_true.presented.as_ref(), when_false.presented.as_ref()) {
            (Some(left), Some(right)) if left == right => Some(left.clone()),
            (None, Some(Type::Reference(_))) if matches!(when_true.kind, ExprKind::Null) => {
                when_false.presented.clone()
            }
            (Some(Type::Reference(_)), None) if matches!(when_false.kind, ExprKind::Null) => {
                when_true.presented.clone()
            }
            _ => None,
        },
        _ => None,
    }
}

/// The type a binary expression's text is presented as, from the types its operands state.
///
/// Arithmetic is JLS 5.6.2's binary numeric promotion over its two operands, which is a rule over
/// the operand *types* and therefore a rule this layer can state exactly when both operands state
/// theirs: `int + long` is a `long`, and a `char`, a `byte` and a `short` operand all promote to
/// `int` — which is the same fact that makes a chain of `iadd`s on them an `int` value. A
/// comparison's own value is a `boolean` (`a < b` is one), and the positions that consume a value
/// never read one: a test is written as the statement it is.
///
/// A `boolean` or reference operand states no arithmetic at all (`b + 1` is not a Java expression,
/// and `a + b` on two references is a string concatenation, not an addition). Bitwise operations
/// accept either two booleans or two integral values; they never turn an int-shaped mixed pair into
/// a Java expression. The layer states **no** type for a shape it cannot write.
fn binary_type(op: BinaryOp, left: &Expr, right: &Expr) -> Option<Type> {
    if matches!(op, BinaryOp::LogicalAnd | BinaryOp::LogicalOr) {
        return matches!(
            (left.presented.as_ref()?, right.presented.as_ref()?),
            (Type::Boolean, Type::Boolean)
        )
        .then_some(Type::Boolean);
    }
    if matches!(
        op,
        BinaryOp::LeftShift | BinaryOp::RightShift | BinaryOp::UnsignedRightShift
    ) {
        integral_promotion_rank(right.presented.as_ref()?)?;
        return Some(if integral_promotion_rank(left.presented.as_ref()?)? == 1 {
            Type::Long
        } else {
            Type::Int
        });
    }
    if matches!(
        op,
        BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessOrEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterOrEqual
    ) {
        return Some(Type::Boolean);
    }
    if matches!(
        op,
        BinaryOp::BitwiseAnd | BinaryOp::BitwiseXor | BinaryOp::BitwiseOr
    ) {
        let left = left.presented.as_ref()?;
        let right = right.presented.as_ref()?;
        if matches!((left, right), (Type::Boolean, Type::Boolean)) {
            return Some(Type::Boolean);
        }
        let left = integral_promotion_rank(left)?;
        let right = integral_promotion_rank(right)?;
        return Some(if left.max(right) == 1 {
            Type::Long
        } else {
            Type::Int
        });
    }
    let left = promotion_rank(left.presented.as_ref()?)?;
    let right = promotion_rank(right.presented.as_ref()?)?;
    Some(promoted(left.max(right)))
}

/// The integral promotion rank for `&`, `^` and `|` (JLS 15.22).
fn integral_promotion_rank(ty: &Type) -> Option<u8> {
    match ty {
        Type::Byte | Type::Short | Type::Char | Type::Int => Some(0),
        Type::Long => Some(1),
        Type::Boolean | Type::Float | Type::Double | Type::Reference(_) => None,
    }
}

/// Where one operand's type sits in Java's binary numeric promotion (JLS 5.6.2), or `None` for a
/// type no arithmetic takes.
fn promotion_rank(ty: &Type) -> Option<u8> {
    match ty {
        Type::Byte | Type::Short | Type::Char | Type::Int => Some(0),
        Type::Long => Some(1),
        Type::Float => Some(2),
        Type::Double => Some(3),
        Type::Boolean | Type::Reference(_) => None,
    }
}

/// The type one promotion rank stands for.
fn promoted(rank: u8) -> Type {
    match rank {
        0 => Type::Int,
        1 => Type::Long,
        2 => Type::Float,
        _ => Type::Double,
    }
}

/// The operator one field or array assignment writes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssignOp {
    /// `=`
    Assign,
    /// `+=`
    Add,
}

impl AssignOp {
    /// The assignment token this statement writes.
    pub fn spell(self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::Add => "+=",
        }
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
    /// `receiver.name = value;` or `name = value;` — the write a synthetic accessor's call
    /// performed, or one a `putfield`/`putstatic` of the presented body performed where `field@1`
    /// proved the member it writes. A missing receiver is only used for a same-class blank static
    /// final write whose declaration proof makes the simple name unambiguous.
    ///
    /// A write accessor returns nothing, so the call site that used to spell it is a statement:
    /// the receiver is the first argument the site passed, and the value is the second. A `putfield`
    /// reaches the same node with the value and the receiver the instruction itself read, and a
    /// `putstatic` with the owner type as the receiver.
    FieldAssign {
        receiver: Option<Expr>,
        name: String,
        op: AssignOp,
        value: Expr,
    },
    /// `array[index] = value;` — one element written (P3 2b).
    ///
    /// An element write is a statement and not an assignment to a name: [`StmtKind::Assign`] writes
    /// a local's name, and `a[i] = v` writes a location the bytecode names with three values (the
    /// array, the index and the value). The array is spelled exactly as it is in the read
    /// ([`ExprKind::Index`]), so `a[i][j] = v` is this node over an `Index`, and the index and the
    /// value are the expressions their own producers write.
    ///
    /// The value meets the element type the **array's** own type states — not a type this node
    /// carries: the check is [`crate::build`]'s, taken where the instruction's operands are read,
    /// and the node is only the text (`array[index] = value;`).
    IndexAssign {
        array: Expr,
        index: Expr,
        op: AssignOp,
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
    /// `break;` — an edge whose target is the active loop's proven exit.
    Break { label: Option<String> },
    /// `continue label;` — the target is a proved enclosing loop's header or `for` update.
    Continue { label: Option<String> },
    /// `throw <expr>;` — the exception expression evaluated by an ordinary `athrow`.
    ///
    /// The expression is the value the throw instruction actually reads. Its source type is left to
    /// the expression's own evidence; this node does not infer a `Throwable` hierarchy or insert a
    /// cast the class file did not perform.
    Throw { value: Expr },
    /// `if (<cond>) { … } else { … }`, with an empty `else_body` when the source had none.
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },
    /// `while (<cond>) { … }` — the test runs before every iteration, the body only when it holds.
    While {
        label: Option<String>,
        cond: Expr,
        body: Vec<Stmt>,
    },
    /// `for (init; condition; update)`: the init and update each retain their original statement
    /// anchor even though the header, rather than the body, writes them.
    For {
        label: Option<String>,
        init: Box<Stmt>,
        cond: Expr,
        update: Box<Stmt>,
        body: Vec<Stmt>,
    },
    /// `for (T element : iterable) { … }` after the loop's traversal and element uses are proved.
    ForEach {
        label: Option<String>,
        ty: Type,
        name: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    /// `do { … } while (<cond>);` — the body runs once before the test is read.
    DoWhile {
        label: Option<String>,
        cond: Expr,
        body: Vec<Stmt>,
    },
    /// `switch (<value>) { … }`, with one arm per distinct target of the decoded `switch`.
    Switch { value: Expr, arms: Vec<SwitchArm> },
    /// `try (T n = expr; …) { … }` — the guarded statement the `twr@1` rule proved — or, with no
    /// resources, the `try { … } catch (T n) { … }` the exception table itself states.
    ///
    /// The resources are written in **declaration** order, which is the order the rule read their
    /// initialisations in, and the compiler closes them in the reverse order — the order the bytecode
    /// really closed them in, because that is the order the rule matched the normal path's close
    /// chain against before it may write this node ([`crate::guard`]).
    ///
    /// An **empty** `resources` is what makes this the other statement: there is no header to write,
    /// the `try` is a plain block, and the statement's clauses are the named rows the exception table
    /// states, in table order ([`crate::region::Region::Try`]). Which of the two a node is, is
    /// therefore read off the node itself and never guessed from the clauses beside it — a `try` of
    /// a guarded shape has no clauses, and the one the table states has no resources.
    Try {
        resources: Vec<ResourceDecl>,
        /// The `catch` clauses, in exception-table order. Empty for a guarded statement: a header
        /// with a `catch` beside it is one this build refuses rather than partially presents.
        catches: Vec<CatchClause>,
        body: Vec<Stmt>,
        /// Present only when a guard proved both physical cleanup copies.
        finally_body: Option<Vec<Stmt>>,
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

/// One `catch` clause of a `try`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatchClause {
    /// The class — or the classes, joined with `|` in exception-table order, of a multi-catch — the
    /// clause catches, spelled as Java source (`java.lang.IllegalArgumentException`).
    ///
    /// They are the exception-table rows' own `catch_type`s and no others: the class file states
    /// which classes the handler catches, and widening one to a superclass would claim the handler
    /// catches exceptions the table says it does not, while narrowing it would catch fewer.
    pub ty: String,
    /// The parameter's name. It is the local the handler's own entry store fills, named by the body's
    /// name table — its slot's ordinal (`localN`, `argN`) when the body was compiled without debug
    /// metadata. No name is invented for it.
    pub name: String,
    /// The handler's statements, from its entry to where the code after the `try` begins.
    pub body: Vec<Stmt>,
}

/// One arm of a `switch` statement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwitchArm {
    /// The labels this arm is reached by. Empty means the arm is the no-match case alone.
    pub keys: Vec<i64>,
    /// One proven presentation of the original integer keys. The keys stay as bytecode evidence.
    pub labels: Option<SwitchLabels>,
    /// Whether the no-match case reaches this arm too (a `default:` label beside the keys).
    pub default: bool,
    /// Whether execution falls into the next arm in source order rather than leaving the switch.
    pub fall_through: bool,
    /// The statements of the arm, each of which runs at most once per execution of the switch.
    pub body: Vec<Stmt>,
}

/// A closed choice of case-label spelling. String labels originate only from the same-method
/// String dispatch certificate; enum labels originate only from complete class-source proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SwitchLabels {
    Enum(Vec<String>),
    String(Vec<String>),
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
    fn a_class_literal_expression_presents_the_class_type() {
        let literal = Expr::direct(
            ExprKind::ClassLiteral {
                ty: "java.lang.String".to_owned(),
            },
            0,
        );
        assert_eq!(
            literal.presented,
            Some(Type::Reference("java.lang.Class".to_owned()))
        );
    }

    #[test]
    fn floating_leaves_present_their_own_type_from_their_own_bits() {
        // The type is the leaf's own, stated by its shape: a float literal never widens into a
        // double, and the bits it carries decide nothing about the type.
        let float_leaf = Expr::direct(ExprKind::Float(0x3f80_0000), 0);
        let double_leaf = Expr::direct(ExprKind::Double(0x4000_0000_0000_0000), 0);
        assert_eq!(float_leaf.presented, Some(Type::Float));
        assert_eq!(double_leaf.presented, Some(Type::Double));
    }

    #[test]
    fn the_subset_spells_the_operators_and_types_it_claims() {
        assert_eq!(BinaryOp::Remainder.spell(), "%");
        assert_eq!(BinaryOp::NotEqual.spell(), "!=");
        assert_eq!(BinaryOp::BitwiseAnd.spell(), "&");
        assert_eq!(BinaryOp::BitwiseXor.spell(), "^");
        assert_eq!(BinaryOp::BitwiseOr.spell(), "|");
        assert_eq!(BinaryOp::LogicalAnd.spell(), "&&");
        assert_eq!(BinaryOp::LogicalOr.spell(), "||");
        assert_eq!(
            Type::Reference("java.lang.String".into()).spell(),
            "java.lang.String"
        );
        assert_eq!(Type::Long.spell(), "long");
    }

    #[test]
    fn bitwise_types_require_two_booleans_or_integral_operands() {
        let expression = |ty| Expr::direct(ExprKind::Integer(1), 1).presenting(ty);
        let ty = |op, left, right| binary_type(op, &expression(left), &expression(right));

        assert_eq!(
            ty(BinaryOp::BitwiseAnd, Type::Byte, Type::Char),
            Some(Type::Int),
            "byte and char undergo unary integer promotion"
        );
        assert_eq!(
            ty(BinaryOp::BitwiseXor, Type::Long, Type::Short),
            Some(Type::Long),
            "a long operand retains the long result width"
        );
        assert_eq!(
            ty(BinaryOp::BitwiseOr, Type::Boolean, Type::Boolean),
            Some(Type::Boolean)
        );
        assert_eq!(
            ty(BinaryOp::BitwiseAnd, Type::Boolean, Type::Int),
            None,
            "JVM int-shaped mixed values have no direct Java bitwise expression"
        );
        assert_eq!(
            ty(BinaryOp::BitwiseXor, Type::Int, Type::Float),
            None,
            "floating values are not integer bitwise operands"
        );
    }

    #[test]
    fn logical_operators_require_boolean_operands() {
        let boolean = Expr::direct(ExprKind::Boolean(true), 1);
        let integer = Expr::direct(ExprKind::Integer(1), 2);
        assert_eq!(
            binary_type(BinaryOp::LogicalAnd, &boolean, &boolean),
            Some(Type::Boolean)
        );
        assert_eq!(
            binary_type(BinaryOp::LogicalOr, &boolean, &boolean),
            Some(Type::Boolean)
        );
        assert_eq!(binary_type(BinaryOp::LogicalAnd, &boolean, &integer), None);
        assert_eq!(binary_type(BinaryOp::LogicalOr, &integer, &integer), None);
    }

    #[test]
    fn shift_result_width_comes_only_from_integral_left_operand() {
        let expression = |ty| Expr::direct(ExprKind::Integer(1), 1).presenting(ty);
        let ty = |op, left, right| binary_type(op, &expression(left), &expression(right));
        for op in [
            BinaryOp::LeftShift,
            BinaryOp::RightShift,
            BinaryOp::UnsignedRightShift,
        ] {
            for left in [Type::Byte, Type::Short, Type::Char, Type::Int] {
                assert_eq!(ty(op, left, Type::Long), Some(Type::Int));
            }
            assert_eq!(ty(op, Type::Long, Type::Int), Some(Type::Long));
            assert_eq!(ty(op, Type::Int, Type::Boolean), None);
            assert_eq!(ty(op, Type::Boolean, Type::Int), None);
            assert_eq!(ty(op, Type::Float, Type::Int), None);
        }
    }
}
