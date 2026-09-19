# P4 modern-fact fixtures

Real compiled samples for the modern structural facts (`tests/p4_modern_facts.rs`, P4 1.2). The
sources live in `src/`, the outputs in one directory per target release, and every output is a
plain `javac` product: no test needs the compiler at run time.

Compiler: `javac 23.0.1` (`/usr/bin/javac`), on macOS, run from this directory.

Commands:

```text
javac --release 16 -g:none -d v16 src/RecordSample.java src/Marker.java
javac --release 17 -g:none -d v17 src/SealedSample.java src/Alpha.java src/Beta.java \
      src/NestSample.java src/ConcatSample.java src/module-info.java src/p4/sample/Provider.java
javac --release 8  -g:none -d v8  src/ConcatJava8.java
```

`javac --release 8` prints the two deprecation warnings JDK 23 gives that option, and the module
name `p4.sample` gets the "module name component should avoid terminal digits" warning. Both are
warnings about the *options and the sample*, not about the outputs, and neither changes a byte of
them.

## Outputs

| output | major | bytes | SHA-256 |
| --- | --- | --- | --- |
| `v16/Marker.class` | 60 | 340 | `9be43663a36c95acdbff09208fe1edf2d1910987e4b2df85a09b1d08cc527adc` |
| `v16/RecordSample.class` | 60 | 1245 | `84637ae2bd80b3581e2250bf1e00ef85f64f5eb79dda5bb076514f1d651c2f1c` |
| `v17/Alpha.class` | 61 | 140 | `2917583da36083fc161ffc2ca4a2304a6894491f55b5cca107aab73b40d0216f` |
| `v17/Beta.class` | 61 | 139 | `96c153358d48524ccab3a4bb943d4b56f6825944c86473503c67b13480003d42` |
| `v17/ConcatSample.class` | 61 | 884 | `96bf68a9c56377e8d53cb73225fdccf9b5b77c62acd4dce3fa1ff5ebc36d0b27` |
| `v17/module-info.class` | 61 | 177 | `ee6b90ed7e4c2c419d0bd783ab3a4e8f78504e410f7e91aa56361d12168f11a2` |
| `v17/NestSample.class` | 61 | 247 | `d3ac955d0ee802f2a0511b2a607492b0d0ab9741616a643f45039be5b8ce5bfb` |
| `v17/NestSample$Inner.class` | 61 | 370 | `b5412c0316274866a49ebbc00398d1a7fc5dd7a5ea9970ed59762ebf6652b582` |
| `v17/p4/sample/Provider.class` | 61 | 335 | `23a56447fdf5ffabd2360dd4a29ea6a3bde68578f2b96181c2be900fc3a825b4` |
| `v17/SealedSample.class` | 61 | 119 | `9def347454fa01d789721cfff8d6512950b7cdcfd754f005fa5ac177d9b5f59e` |
| `v8/ConcatJava8.class` | 52 | 1087 | `a9e15c5a392a2f6acd6c56510a8c43e4a62d97314a5f7d6b710a751176262bf2` |

What each output carries, as `javap -v -p` reports it:

* `v16/RecordSample.class` — the `Record` attribute with three components (`count:I`, `label:Ljava/lang/String;`,
  `stamp:J`); the `label` component carries its own attribute (`RuntimeVisibleAnnotations` for
  `@Marker`), which is the nested `attribute_info` list the reader reports as shells. Three
  `invokedynamic` sites bootstrapped by `java/lang/runtime/ObjectMethods.bootstrap` — deliberately
  **not** string concat, so the concat classifier is tested against a class full of indy sites it
  must not claim. This is the file whose two version fields the release tests patch.
* `v16/Marker.class` — the annotation type `MARKER` targets: `java.lang.annotation.Target`
  `RECORD_COMPONENT` plus `RUNTIME` retention, which is why `javac` writes it inside the `Record`
  attribute rather than on the field or the accessor. (`@Deprecated` does **not**: its targets are
  the field and the method, so it lands outside the `Record` attribute entirely.)
* `v17/SealedSample.class` — `PermittedSubclasses` naming `Alpha` and `Beta`.
* `v17/NestSample.class` / `v17/NestSample$Inner.class` — `NestMembers` on the host and `NestHost` on
  the member, the shapes the nestmate facts are about.
* `v17/module-info.class` — the `Module` attribute with one `uses` and one `provides`.
* `v17/ConcatSample.class` — two `invokedynamic` sites, both `StringConcatFactory.makeConcatWithConstants`
  (one with an `int` and a `String` operand, one with two `String` operands), each with a recipe.
* `v8/ConcatJava8.class` — the Java 8 output level: `+` compiles to a `StringBuilder` chain and the only
  `invokedynamic` is the lambda's `LambdaMetafactory` site, so this class has no modern fact to conflict with.

## What this compiler does not produce, recorded honestly

* **No `CONSTANT_Dynamic`.** `javap -v` reports zero `Dynamic` entries for every output above, for
  `javac 23.0.1` at `--release 8`, `16` and `17`. The constant-dynamic graph therefore has no real
  compiled sample: its fixtures are hand-built in `tests/p4_modern_facts.rs` (`condy_fixture`), the
  same convention `tests/p1_xref_bootstrap.rs` already uses, and for the same reason — the graph
  pins exact constant-pool indexes, bootstrap indexes and argument positions that a compiler cannot
  be asked to place.
* **No `makeConcat`.** Both concat shapes in `src/ConcatSample.java` compile to
  `makeConcatWithConstants`; `javac` did not emit the recipe-free `makeConcat` entry for either. The
  site's strategy is still read from the site's own name, so the recipe-free case is covered by the
  classifier's `Other`/`Concat` arms rather than by a compiled sample.
