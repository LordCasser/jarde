# Post-change member-annotation replay

The implementation CLI used here is `target/release/jarde-cli`, SHA-256
`8f48a9b418385b20d63257d707e0a54ef833c21b9b66901108fdcce865ea9169`. Both frozen replay scripts
were copied to a temporary evidence directory before running; the checked-in `generated/` baseline
was left untouched. [`replay-script.diff`](replay-script.diff) records the only edits to those
copies: the CLI hash, and the 1.1 Jarde output expectation from the old missing-annotations result
to the recovered result.

The 1.1 replay passed with original, JADX, and Jarde source each compiled using
`javac --release 8` and loaded under `java -Xverify:all`. All three runners printed the same four
lines: `true`, `true`, `1`, `5`. The frozen input class remained 362 bytes with SHA-256
`63fbb2c76199125f23540adc0611e65232be055e6817899291722c90d301c996`.

The 1.2 boundary script passed with the rebuilt CLI. The original legal fixture compiled and
verified with output `7 / 7 / 0 / 3`; the two controlled classfile patches also passed `javap` and
`-Xverify:all`. The new Jarde text and JSON preserve the field, method, and all three descriptor
parameter-position uses. JSON for the duplicate patch retained both encoded facts and refused both
spellings; JSON for the parameter-count patch retained the independent method annotation, recorded
the raw count of two against three descriptor positions, and refused all parameter groups. The
legal boundary JADX sources, including the annotation type, also compiled with `--release 8`,
passed `-Xverify:all`, and printed `7 / 7 / 0 / 3`.

The full boundary Jarde class still cannot compile because its pre-existing `wideAndVarargs` body
has no recovered `return` statement. I generated the complete class source with both the frozen
pre-change CLI (SHA-256
`f2fb24241c2692dff53060576e185eb282617a6881e81900140bb2eb62006544`) and the rebuilt CLI, then
compiled each generated class set without editing the output. Both fail only with javac's
“missing return statement” at that method. The member-annotation change adds declaration text and
facts but does not claim this body is recovered or that this boundary class compiles.

To verify that runtime-invisible values survive recompilation without that unrelated body limit,
[`verify_invisible_smoke.py`](verify_invisible_smoke.py) generates a temporary, compileable class
with an annotated field, method, and `long`/`double`/varargs parameters. It passed Java 8 compilation
and `-Xverify:all` both before and after Jarde source generation; both runs printed `7`. The original
and recompiled `BoundarySmoke.class` are byte-for-byte identical (SHA-256
`842b3757521624079c92f19fda43aa96d1673ca662db5b2a9c9e87c59cd3adf9`). `javap -v` confirms the
field/method `RuntimeInvisibleAnnotations` and all three `RuntimeInvisibleParameterAnnotations`
groups retain values 1 through 5.

The full replay outputs used for this verification were kept in the temporary copy at
`/var/folders/gm/lzn3ylzx4lz7mx29lftngdz40000gn/T/jarde-member-ann-replay-ezmc75o7/member-annotation-uses`.
