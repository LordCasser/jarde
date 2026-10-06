# recover-static-generic-field-init-text — frozen fixtures

The generic-static-field-init patrol's own `MN`/`RG` shapes
(`openspec/evidence/java-syntax-2026-10-05/generic-static-field-init-patrol/fixture/`) and this
change's two controls, compiled by both javac legs. `tests/recover_static_generic_field_init_text.rs`
reads exactly these bytes.

## Sources

`MN` and `RG` are the patrol's fixtures **verbatim** (`cmp`-identical to the patrol's own copies):

- `MN` — the patrol's anchor: two static generic fields whose `Signature` projection is refused
  (`f1` diamond, `f2` explicit type argument), the instance field `f3` the change's zero-regression
  control rides on, and a `main` that prints `f1.v + f2.v + f3.v`;
- `RG` — the patrol's nested form: `field` (diamond static field), `names` (a generic static field
  whose initializer names no folded class), `nested` (a doubly nested diamond, the shape whose
  broken text ate two argument prefixes), plus the `pick`/`newInner` method refusals this change
  does not touch.

`SG` and `P1` are this change's additions, and they exist because the patrol's two anchors cannot
settle the change's second half alone:

- `SG` — the same shape as `MN` (a static generic field whose `Signature` projection is refused, its
  initializer inlined into the declaration) written so that **nothing else** in the class is
  refused: the nested `Hold<T>` stores an `Object`, so its constructor keeps the erased descriptor
  the renderer spells and the whole-class presentation compiles as one unit. Its `main` prints
  `f1.v + f2.v + f3.v` = `ab5`, exactly what `MN` prints, so the compile-and-run leg of the
  acceptance is measured on the same values;
- `P1` — the mechanism's non-generic control: a static field of a **non-generic** nested class whose
  initializer is inlined into the declaration. The broken text is a property of the fold's
  re-spelling of a declaration that names one folded class twice, not of generics: at the change's
  parent commit `P1` renders `static Box b = new P1$Box;` (the whole argument list eaten) and
  `MN`/`RG`/`SG` render their `…$Holdava.lang.Object)` forms. `P1` also compiles and prints `1` on
  both legs after the change.

| source | SHA-256 |
| --- | --- |
| `MN.java` | `29ba75814a77df6c49af2f5d69de27ee0479e33376665b720877da2bd4a082ce` |
| `P1.java` | `dc0732c96d80c0d3a0d4d625531432914b2ba5d79165dfd274182671407fb5ff` |
| `RG.java` | `32852ae7ca176a40a101f56e619a00433b73e324938738545bb4fad734a254f0` |
| `SG.java` | `a4ea63d3e9c0392a9a58107c19a34d5c07b3a141d5f9216d04504bceef89f9a9` |

## The two legs (one source, two compilers)

- `v8/` — **javac 23.0.1**, `javac --release 8 -Xlint:-options -d v8 *.java`;
- `v8-javac8/` — **real javac 8**, Corretto 1.8.0_432
  (`/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac`,
  `javac -version` → `javac 1.8.0_432`, **no `--release`**: the default target is Java 8),
  `javac -d v8-javac8 *.java`.

The `v8` leg's four class files of the patrol family are `cmp`-identical to the patrol's own frozen
fixture (`MN.class` 1,149 bytes, `MN$Hold.class` 395, `RG.class` 1,966, `RG$Hold.class` 483 — the
same SHA-256 the patrol's `fixture/sha256.txt` records), so the patrol's record and this leg are the
same bytes.

| class file | bytes | SHA-256 |
| --- | ---: | --- |
| `v8/MN.class` | 1,149 | `174eb6b15bc9360756178e4090885acd7755d6dea3b4d4ac5fb8751deba3a00a` |
| `v8/MN$Hold.class` | 395 | `7ad1949b336320cbeee87b99195cbfecbfbb89b738398ecd284692e5ba6f77b2` |
| `v8/P1.class` | 546 | `34ea1289a3b36cb0b4d5b26d45faa40f0d00abe60d56de5a16367f99af2a0fbe` |
| `v8/P1$Box.class` | 265 | `d119faec00a6bb298baa5a701d0544039b07924302500551d89243c21aae274f` |
| `v8/RG.class` | 1,966 | `dec8bdc4721ccf96a192024318eb09a684dba7fd44d4407c7f52fdb66ed4148f` |
| `v8/RG$Hold.class` | 483 | `bc0083342da5c6e0a296f17828bcc5f6591606a5293009e8c354f4cfb7471bdd` |
| `v8/SG.class` | 1,132 | `c078e7a115935609f4060b8f19784072408a7bdc08e4135e767ea2cbb7ed86e1` |
| `v8/SG$Hold.class` | 364 | `8cb13c843d30244beb6d9e95464f70043485da78bdd31341a31748341580168a` |
| `v8-javac8/MN.class` | 1,149 | `96c0b6aecd58d5702abbc2366597456aa0c486c7c74bc21b201320afe1691aed` |
| `v8-javac8/MN$Hold.class` | 395 | `8c82a5236f4839601bb69e45f85d0cbb5bce7649b14b3c482ecc1aeaf0ace79e` |
| `v8-javac8/P1.class` | 546 | `8adffde0e8aad0f6311918fb4ff56baaf6f29be6d9c602c54223b526967c402d` |
| `v8-javac8/P1$Box.class` | 265 | `6a480118d39fb3c8dfa433ce03fdba7355c583aa0daef160bbe4e1066eaa83b2` |
| `v8-javac8/RG.class` | 1,969 | `51c8010d7ce531aadd2a964b02c87e8a5f403fab20f9c33eddfcf41f8ba23ee7` |
| `v8-javac8/RG$Hold.class` | 483 | `d3b135b072bf52c633b3904985e6d8403f89395817fbdf064a256323a313d2a7` |
| `v8-javac8/SG.class` | 1,132 | `18a23b7bd50f02b6a23c3d13d4b43491449ac254d84ac810fb1b43d0c37c93d3` |
| `v8-javac8/SG$Hold.class` | 364 | `97851e9aafdf4c97288bd37f6d805a35aa110e15f79a134aa3db98f32b8ce51a` |

## The discriminating facts

The render of each class is the **family-fold** posture (`--policy plain-jar` over a jar holding
the class and its `$`-named companion), because that is the posture the patrol's broken text was
recorded in: the fold re-spells every reference to a folded member with the source nesting its
declaration states, and the field declarations carry the initializer the `<clinit>` proof inlined
into them.

At the change's parent commit `dccd21c3` the fold renders (both legs, byte-identical):

| class | rendered field declaration | `javac --release 8` |
| --- | --- | --- |
| `MN` | `static Hold f1 = new MN$Holdava.lang.Object) "a");` | exit 1, `需要'('或'['` |
| `RG` | `static Hold nested = new RG$Holdava.lang.Object) new RG$HolHold.lang.Object) java.lang.Integer.valueOf(5)));` | exit 1 |
| `SG` | `static Hold f1 = new SG$Holdava.lang.Object) "a");` | exit 1 |
| `P1` | `static Box b = new P1$Box;` | exit 1 |

After the change the same four declarations are `new Hold((java.lang.Object) "a")`,
`new Hold((java.lang.Object) new Hold((java.lang.Object) java.lang.Integer.valueOf(5)))`,
`new Hold((java.lang.Object) "a")` and `new Box(1)`.

`MN`'s and `RG`'s whole-class presentations still do **not** compile, and that is not this change's
text: with the parse error gone, the only remaining error in `MN` is the nested `Hold<T>`'s own
erasure pair — `T v;` (its field `Signature` is projected) beside `Hold(java.lang.Object arg1)`
(its constructor's `Signature` is refused, `ordinary_generic_source_unproved`), so `this.v = arg1;`
is `Object` into `T`. That pair is the projection domain's own open item, not the field-initializer
text's, and it is present in the class's single-class presentation too. `SG` and `P1` are the
fixtures that carry the change's compile-and-run leg for that reason.

## The tests that read them

`tests/recover_static_generic_field_init_text.rs`:

- not ignored: both legs render every fixture once and pin the field declarations whole — the four
  re-spelled declarations, `MN.f3`'s untouched instance declaration, the refusal markers'
  preservation, and the absence of every residue of the broken text (`Holdava`, a folded `$` name in
  a declaration line, a truncated `new X$Y;` initializer);
- ignored (`cargo test -- --ignored`, needs a JDK on `PATH`): `SG`'s and `P1`'s whole-class
  presentations are stripped (comment lines dropped), compiled by the installed `javac --release 8`
  **and** by a real javac 8 when one is present (`JARDE_JAVAC8`, else the Corretto path above), run
  with `-Xverify:all` and compared with the fixture's own class run (`ab5` and `1`). `MN`'s and
  `RG`'s runs are compared the same way after the one out-of-scope erasure pair is hand-corrected
  (`Hold(java.lang.Object arg1)` → `Hold(T arg1)`), which is what makes their `ab5` and `y7insf5`/`2`
  answers measurable without this change touching the projection domain.
