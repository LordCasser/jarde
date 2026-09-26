# Conditional boolean field stores

`ConditionalFieldWrites.class` is Java 8 `javac` output. Its `putStatic(Z)V` and
`putInstance(Z)V` methods put the respective conditional's integer result directly into
`staticFlag:Z` via `putstatic` and `instanceFlag:Z` via `putfield`. The permanent Rust replay
test checks that Jarde preserves both arms and the boolean store conversion.

`putInteger(Z)V` writes the same `2`/`3` conditional to an `int` field and is the negative
descriptor control: Jarde must retain the integer conditional without boolean normalization.

`ConditionalFieldWritesNon01.class` is made by `patch_non01.py`: it changes only the two
conditional producers from `iconst_1`/`iconst_0` to `iconst_2`/`iconst_3`. The branch and field
consumer instructions are unchanged. The JVM observes 2 as false and 3 as true when storing to a
boolean field. Both class files pass `java -Xverify:all`.

Rebuild the frozen original and patched classes with:

```sh
FIXTURE=tests/fixtures/p3-conditional-values/field-writes
TMP=$(mktemp -d)
javac --release 8 -g:source,lines,vars -d "$TMP" "$FIXTURE/ConditionalFieldWrites.java"
cmp "$TMP/ConditionalFieldWrites.class" "$FIXTURE/ConditionalFieldWrites.class"
python3 "$FIXTURE/patch_non01.py"
```

`Runner.java` prints both selectors for the two field stores. The original class prints
`false:false` then `true:true`; the patched class prints `true:true` then `false:false`.
`openspec/changes/recover-conditional-values/evidence/field-writes/replay.py` repeats the
original/Jarde recompile and execution comparison, and records that JADX 1.5.6's patched source
does not compile because it assigns `int` conditional expressions to `boolean` fields.
