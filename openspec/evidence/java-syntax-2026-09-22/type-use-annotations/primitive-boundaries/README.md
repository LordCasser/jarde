# Primitive scalar type-use annotation boundary

This fixture isolates a classfile state that Java source with the same annotation
declaration and the ordinary `@A int` spelling does not reproduce. The annotation
permits both declaration and type-use targets. Java 8 source therefore puts each
annotation on a primitive field, primitive return type, and primitive parameter
into both the declaration-annotation attribute and `RuntimeVisibleTypeAnnotations`.
The patch removes only the former member attributes. It leaves each type annotation,
all code, and the annotation declaration intact.

## Replay

From this directory, run:

```sh
sh replay.sh
```

The script compiles with `javac --release 8`, patches only the generated
`ScalarCases.class`, inspects both classes with `javap -v -p`, and starts reflection
checks with `java -Xverify:all`. `patch_scalar_class.py` is a small classfile
attribute-table rewriter; it reports exactly which member attributes were removed
and hashes the raw `Code` attribute payloads before and after. Generated classfiles
and reports are in `build/`.

## Result

See [summary.md](summary.md) for the observed physical attributes, reflection
results, hashes, and the source-expression limit established by this fixture.
