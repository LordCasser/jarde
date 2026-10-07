# 03 · Implementation (tasks 2.1 and 2.2)

## The edit

One function and one call site, both in `crates/jarde-java/src/build.rs`
([implementation.diff](implementation.diff)):

- **`widen_covariant_store_receiver(array, component, value)`** — new, placed beside
  `array_element`/`written_type` (the array-channel readers). It answers the receiver unchanged
  unless the component is a `Type::Reference` that is neither `Object` nor `java.lang.Object`, the
  value states a `Type::Reference`, and that reference is not the component; when all three hold it
  wraps the receiver in `ExprKind::Cast { ty: Reference("java.lang.Object[]"), value: receiver }`
  carrying the receiver's **own** origin set (no new BCI is claimed — a presentation decision, the
  same discipline `cast_argument`/`integer_low_bit_boolean` keep);
- **`array_write`** renders the receiver into `mut array` and, in the `Some(ty)` arm where the
  element type the array's own facts state meets the value, hands it to that function. The arm is
  the store's own type check (`meeting_position`, `Widening::Position`); the primitive
  narrow-store arm (`cast_argument`) and the boolean arm above it are untouched, and the
  compound-update path (`a[i] += v`, which proves a numeric component) returns before this point.

Nothing else changed in the layer: `array_initializer_element`'s reference rule, `array_element`,
`element_receiver_type`, the read-side proofs (`recover-array-element-field-receiver`), the
withdrawal/fallback paths and the statement emitter are all byte-identical. The cast's rendering
(`((java.lang.Object[]) local0)[0] = …`) is the existing emitter's own: a cast binds `UNARY`, so the
indexee position groups it once.

## The rule, stated exactly

*Widen the access receiver of one array store when the component type the store's own facts state is
a reference type other than `Object`, and the stored value's presented type is a reference type that
is not that component.*

The proven-compatible set is the closed rule the array initializer's element rule already states —
the exact component type, an `Object` component, a `null` value — because **which class is assignable
to which is a subtype judgment this layer deliberately does not make**. Everything else is a
reference value the presented component cannot be *proven* to accept, and the widened text is the
presentation that keeps both halves of the contract:

- the text compiles: every reference array is a subtype of `Object[]` (JLS 4.10.3), so the cast is a
  widening reference conversion applied to an expression whose own static type is that array;
- the runtime check is unchanged: `(Object[]) x` performs no check of its own for a reference array,
  the receiver is evaluated exactly once, and the `aastore` still tests the value against the
  array's **runtime** component — the `ArrayStoreException` fires exactly where the class file
  throws it;
- the declaration is not rewritten: the local keeps the array its own facts state, so a read of the
  same local is presented as it was.

## Why the admission looks wider than "incompatible"

`SC.subtypeStore` (`CharSequence[] cs = new CharSequence[2]; cs[0] = "s";`) widens, because `String`
being assignable to `CharSequence` is exactly the subtype judgment the layer does not make. The
alternative — a hierarchy fact — is a different slice's design question (the changelog names the
platform-hierarchy channels that would be its precedent); it is not something this change invents.
The consequence is behavior-free: the text still compiles, the class still answers what it answered
(both legs), and the fixture pins the boundary so a later hierarchy proof can narrow it deliberately.

## The shapes the change does not touch

| shape | why it is outside the admission |
| --- | --- |
| primitive component (`int[]`, `byte[]`, …) | primitives have no covariant arrays; the store's own conversion rules own that side |
| `Object` component | any reference value is proven compatible with it |
| same type | the value's own type is the component |
| `null` value (also a lambda, a method reference) | the value states no type at all, so no mismatch is provable — the rule every other position keeps |
| a receiver whose component no fact states (a two-branch local, a merge) | `array_of_value` answers nothing; nothing is guessed, so no widening (`UB.merged` keeps its text, debt and all) |
| a receiver that is not a proven reference array | same: no component fact, no widening |
| the read side (`xs[i]`, `xs[i].f`, `local0[0][0] = …` reads) | untouched: this change only writes a store's own access receiver |

## Tests (task 2.2)

`tests/recover_covariant_array_store_receiver.rs` — four pinned-presentation tests and four ignored
replays, all on both compiler legs (javac 23.0.1 `--release 8` and the real Corretto 1.8.0_432,
whose path the test asserts before use). The fixtures, their digests and the discriminating facts
are [04-fixtures.md](04-fixtures.md) and
`tests/fixtures/recover-covariant-array-store-receiver/README.md`.

- the patrol anchor: `AS`'s whole presentation is pinned, `storeWrong`'s store statement is asserted
  to keep BCIs 5/6/8/11 as anchors (the `aastore` at 11 included), and the same-type control is
  asserted present verbatim;
- the driver: `SD`'s whole presentation is pinned — the three widened stores beside the read-back,
  the two proven-compatible controls, the primitive-array negative and the same-type element
  receiver;
- the negatives: `UB`'s whole presentation is pinned **per leg** (the real javac 8 writes the
  `checkcast` twice where javac 23 folds it once — a leg difference of the pool, not of this change);
- the boundary: `SC`'s whole presentation is pinned;
- the replays: `AS` answers `s`/`ASE1`/`ASE2` on both sides of each leg; `SD`'s driver prints each
  covariant store's exception **type** and **throwing method** and the two runs agree exactly; `SC`
  answers `s`; and `UB`'s stripped text must stay uncompilable (its merged local's debt, which this
  change does not touch — a run that compiles it fails the assertion loudly).

The read-side suites (`recover_array_element_field_receiver`, the array-store/array-type families)
are green in the workspace run: the whole-corpus two-leg render scan shows **zero** deltas outside
this change's own fixtures ([05-corpus-sweep.md](05-corpus-sweep.md)), which is the stronger form of
the same statement.
