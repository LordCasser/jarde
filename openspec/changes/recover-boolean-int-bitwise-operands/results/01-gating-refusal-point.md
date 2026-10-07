# 1.1 — the refusal emitter and the operand-presentation carrier

All line numbers are of this change's worktree at the implementation commit
(`crates/jarde-java/src/build.rs` unless stated otherwise), read with `grep -n` on the frozen
files. Every quoted excerpt is the file's own text.

## The refusal emitter — one site

```text
$ grep -n "which no Java integral or boolean bitwise expression accepts" crates/jarde-java/src/build.rs
23943:                                "the bitwise operator `{}` at BCI {bci} has operands presented as `{left_type}` and `{right_type}`, which no Java integral or boolean bitwise expression accepts",
```

It is the one place that diagnostic exists, inside `Builder::render_value`'s
`Operation::Bitwise { op }` arm (the arm starts at the `Operation::Bitwise { op } =>` match of
`render_value`), and it fires exactly when the expression node it just built states **no** type:

```rust
let expression = Expr::direct(ExprKind::Binary { op: bitwise_op(*op), left: …, right: … }, bci);
if expression.presented.is_none() {
    return Err(format!(
        "the bitwise operator `{}` at BCI {bci} has operands presented as `{left_type}` and `{right_type}`, …",
```

## The operand-type carrier — `Expr::presented`, computed by `ast::binary_type`

`Expr::direct` computes `presented` from the node's own shape (`ast.rs:549 presented_of`), and for a
binary node that is `ast.rs:633 binary_type`. Its bitwise arm is the rule the refusal reads:

```text
$ grep -n "BinaryOp::BitwiseAnd | BinaryOp::BitwiseXor | BinaryOp::BitwiseOr" crates/jarde-java/src/ast.rs
665:        BinaryOp::BitwiseAnd | BinaryOp::BitwiseXor | BinaryOp::BitwiseOr
```

```rust
if matches!(op, BinaryOp::BitwiseAnd | BinaryOp::BitwiseXor | BinaryOp::BitwiseOr) {
    let left = left.presented.as_ref()?;
    let right = right.presented.as_ref()?;
    if matches!((left, right), (Type::Boolean, Type::Boolean)) { return Some(Type::Boolean); }
    let left = integral_promotion_rank(left)?;   // Boolean => None
    let right = integral_promotion_rank(right)?; // Boolean => None
    …
}
```

So the refusal is stated by the **operands' presentations**, and each operand's presentation comes
from the value it renders: a `boolean` load/parameter/`Z` call/`Z` field/`instanceof`/`baload` of a
`[Z` array is presented `boolean` by [`Builder::boolean_evidence`] (`build.rs:22343`), an `int`
local by its decided type, and a materialised `0`/`1` by the conditional-value fold.

## The evidence rule both sides read — `boolean_proof`

```text
$ grep -n "^struct BooleanProofContext\|^fn boolean_proof\|^enum BooleanEvidence" crates/jarde-java/src/build.rs
511:enum BooleanEvidence { None, Literal, Proven }
29238:struct BooleanProofContext<'a, IsLocal, Visit> { … }
29247:fn boolean_proof<IsLocal, Visit>(…)
```

`BooleanEvidence::Literal` is the `0`/`1` literal (it never seeds a proof on its own);
`Proven` always contains a descriptor or already-decided-local fact; a `Bitwise` node is boolean
only when **both** operands are boolean evidence and at least one of them has a seed.

## Where the materialised `0`/`1` was already folded — `boolean_position_values`

```text
$ grep -n "fn boolean_position_values\|fn bitwise_boolean_operand" crates/jarde-java/src/build.rs
15692:    fn boolean_position_values(&self, proof: &ConditionalValueProof) -> Option<(i64, i64)>
15737:    fn bitwise_boolean_operand(&self, proof: &ConditionalValueProof) -> bool
```

`boolean_position_values` is the one place the two arm constants of a proved conditional value are
read as a boolean (`Some((1, 0))` → the test itself, `Some((0, 1))` → its negation). Its three
positions were, before this change, the `Z` return (`consumer.opcode() == 0xac`), a call argument a
descriptor declares `boolean` (`equality_argument_parameter`) and the field compound
(`bitwise_boolean_operand`, narrowed by `recover-conditional-rhs-field-compound` to a claimed field
write). The change widens the third one — the bitwise consumer — from "the value is taken by a
claimed field write" to "the value's every consumption is a position a `boolean` is read in"
(`build.rs:29368 struct BooleanConsumption`).

## The two shapes, at the bytecode

```text
BW.mix   ([Z)Z:  0: iconst_0; 1: istore_1; … 22: iload_1; 23: iload 5; 25: ixor; 26: istore_1; … 33: iload_1; 34: ireturn
BW.andNot (ZZ)Z: 0: iload_0; 1: iload_1; 2: ifne 9; 5: iconst_1; 6: goto 10; 9: iconst_0; 10: iand; 11: ireturn
```

`mix` refuses because `iload_1` presents `int` (the frame states one `int` shape for the four
int-sized primitives) beside `iload 5`'s boolean `[Z` element; `andNot` refuses because the join's
stack Phi is the materialised `0`/`1`, which no position admitted read as a boolean.
