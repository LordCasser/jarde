# recover-covariant-array-store-receiver — frozen fixtures

The array-covariant-store patrol's own `AS`
(`openspec/evidence/java-syntax-2026-10-05/array-covariant-store-patrol/`) plus the three shapes
this change adds, compiled by both javac legs. `tests/recover_covariant_array_store_receiver.rs`
reads exactly these bytes.

## Sources

`AS` is the patrol's fixture **verbatim** (`fixture/AS.java`, SHA-256
`db31a5d8…b711000` — the same digest as the patrol's own copy): `storeWrong` (the covariant store
`Object[] a = new String[2]; a[0] = Integer.valueOf(1);`), `storeRight` (the same-type control) and
`storeNumber` (the `Number[] ← Integer[]` covariant form), with the patrol's `main` printing the
three readings. The other three are this change's:

- `SD` — the behavior driver. `readBack` (same-type store, read back), `objectComponent` and
  `nullStore` (the two proven-compatible controls: an `Object` component and a `null` value),
  `catchWrong`/`catchNumber`/`catchElement` (the three covariant stores caught as
  `ArrayStoreException`, each printing its exception's **type** and the **method** that threw it),
  `primitiveArray` (the primitive-array negative) and `elementReceiver` (a same-type store whose
  receiver is an array element read);
- `UB` — the unproven-receiver negative: `merged` stores through a local two branches write with
  two different array types (no component fact, so no widening and the member's own declaration
  debt stays), and `throughObject` stores through a `checkcast` receiver whose component the pool
  does state (the same-type case);
- `SC` — the admission's boundary probe: `CharSequence[] cs; cs[0] = "s";` stores a value whose
  type is a **subtype** of the component. No fact in this layer proves that compatibility (the
  subtype judgment is deliberately not made), so this store widens exactly like the patrol's
  covariant shape; the text still compiles and still answers `s`.

| source | bytes | SHA-256 |
| --- | ---: | --- |
| `AS.java` | 1,027 | `db31a5d8b7b9eea60ebd43bfb45f445a2bb69740622fd7168d998cfa0b711000` |
| `SC.java` | 333 | `3722a2b71aca84e757012f5428be50b661e42911bd31848af5af993d87b4ccea` |
| `SD.java` | 2,454 | `953b3f8ad764d08d3d78c70751b7995fb8578b60c92cec3076300ee6c8bf40e3` |
| `UB.java` | 632 | `ef0e9fac7ffafedacf28774b0c1f8395f0f0cee6e9f163da3ca5800a856b9f64` |

## The two legs (one source, two compilers)

- `v8/` — **javac 23.0.1**, `javac --release 8 -Xlint:-options -d v8 *.java` (default debug info:
  no `LocalVariableTable`, which is why the presented texts spell parameters `arg0` and locals
  `localN`, exactly as the patrol's own readings do);
- `v8-javac8/` — **real javac 8**, Corretto 1.8.0_432
  (`/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac`, `javac -version`
  → `javac 1.8.0_432`, **no `--release`**: the default target is Java 8),
  `javac -d v8-javac8 *.java`.

Both legs present byte-identical member texts for every pinned member **except `UB.throughObject`**:
javac 23 folds the checkcast into the receiver's own spelling once, the real javac 8 writes the
declared cast beside the verifier's own (`((java.lang.String[]) arg0)` vs
`((java.lang.String[]) (java.lang.String[]) arg0)`), so that one text is pinned per leg. (`AS`'s
`v8/AS.class` is byte-identical to the patrol's `fixture/as.jar` entry — SHA-256
`2369235f…f286f6`.)

| class file | bytes | SHA-256 |
| --- | ---: | --- |
| `v8/AS.class` | 1,025 | `2369235f5b8fd6002ffe3d44a268b42bc7a13258d8b9f33a672e97ace1f286f6` |
| `v8/SC.class` | 562 | `6fd05406552ed54b00a1a30671e1097f113b59f1525557ded4b0f7eeeb388502` |
| `v8/SD.class` | 2,264 | `827e38584ea16902e52515a526a403be41c106f3dff36f0ce83f2a764c51d152` |
| `v8/UB.class` | 888 | `71fc73a591c7a3559186bbd7904ee507a8aea60b239cb5048228b63cc94637e8` |
| `v8-javac8/AS.class` | 1,028 | `3c1146b23569120148edbae19c7adc99519e3c906f54fc1ffbd2d45ccc485ac8` |
| `v8-javac8/SC.class` | 562 | `16269c197634350176738fe174f26eb906f1cb84e2b22c9bd7f02ecd3e6c196b` |
| `v8-javac8/SD.class` | 2,267 | `9d5a3c109560d9eca35b4cf40405da1f51b8b64783191284dfcab85f31d36fa1` |
| `v8-javac8/UB.class` | 891 | `22872ba4c12a01ff4cdd1c193a58dcc9874e24fdd48aba0419e76bfda323c274` |

## The discriminating facts (javap, `v8` leg)

`AS.storeWrong` — `1: anewarray java/lang/String; 4: astore_0; 5: aload_0; 6: iconst_0;
7: iconst_1; 8: invokestatic Integer.valueOf; 11: aastore`: the frame types the local `String[]`
(no LVT states the source's `Object[]`), and the `aastore` at BCI 11 is the store the presented
text refuses. The store statement's own anchors are 5, 6, 8 and 11.

`AS.storeNumber` — the same shape with `anewarray java/lang/Integer` and `Double.valueOf`, so the
widening is not a `String`-only rule.

`SD.catchElement` — `2: multianewarray [[Ljava/lang/String;; 9: aaload; 15: aastore`: the store's
receiver is the **element read** `local0[0]`, and it is the element's own array type that states
the component. `SD.primitiveArray` — `newarray int` and `iastore`: a primitive component has no
covariant arrays and never widens. `SD.readBack`/`objectComponent`/`nullStore`/`elementReceiver` —
the store the same-type rule, the `Object` component, the `null` value and the element-receiver
control keep.

`UB.merged` — `1: ifeq 9; 6: goto 11`: the local is written by two branches with `String[]` and
`Integer[]`, so the merge states no component and `array_of_value` answers nothing: no fact, no
widening, and the member's own declaration debt (`java.lang.String[] local3; … local3 = arg1;`)
stays as it was. `UB.throughObject` — `checkcast java/lang/String[]` before `aastore`: a component
the pool does state, storing its own type.

`SC.subtypeStore` — `1: anewarray java/lang/CharSequence; … ldc "s"; aastore`: the component is a
supertype of the value the descriptor states, which this layer cannot prove (it makes no subtype
judgment), so it widens too.

## The tests that read them

`tests/recover_covariant_array_store_receiver.rs`:

- not ignored: both legs render every fixture once and pin each class's whole presentation (the
  widened receivers, the controls and negatives, the `AS` store statement's BCIs 5/6/8/11 as
  anchors of the widened statement), plus the envelope's self-header;
- ignored (`cargo test -- --ignored`, needs both JDKs): every stripped text is compiled with
  `javac --release 8` **and** the real javac 8 and run under `-Xverify:all` beside the fixture's
  own class files — `AS` answers `s`/`ASE1`/`ASE2` on both sides, `SD` answers each covariant
  store's exception type and throwing method identically, `SC` answers `s`, and `UB`'s stripped
  text must stay uncompilable (the merged local's own debt, which this change does not touch).
