# recover-write-accessor-field-types — frozen fixtures

Every `.class` here was produced by **real javac 8** (Corretto 1.8.0_432) unless a section below
documents an equal-length byte patch. The dual leg (same sources, `javac --release 8` on the
toolchain's JDK) is compiled in-test, never frozen.

Compiler used for every frozen leg:

```text
/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
javac 1.8.0_432
```

Command line for every family below: `javac -d <outdir> <Source>.java` (no `--release` — real
javac 8 is the leg this change's anchors are frozen from).

## wa/ — main anchor (the nine write-accessor types, byte-identical copy)

`WA.class` and `WA$S.class` are **byte-identical copies** of the frozen patrol fixture
`openspec/evidence/java-syntax-2026-10-04/value-returning-write-accessor-patrol/fixture/WA/`
(SHA256 below — do not recompile replacements; that breaks the baseline comparability the earlier
measurements preserved). `WA.java` is that fixture's own source, copied alongside.

`WA` declares nine private fields of nine different types (`boolean`/`int`/`long`/`double`/
`String`/`byte`/`short`/`char`/`float`) and the inner class `S` writes one of them per setter, so
real javac 8 synthesises nine **value-returning** write accessors `access$002 … access$802`
(`aload_0; <xload_1>; dup_x1|dup2_x1; putfield; <xreturn>`). Measured per type in
`results3/WA-javap-code.txt` of that patrol: single-slot rows stack/locals 3/2, `long`/`double`
rows 5/3 with `dup2_x1`.

Behavior baseline of the original classes (the `main` line the nine setters print): `true123.0x45c6.0`.
The frozen rendering legs live in
`openspec/evidence/java-syntax-2026-10-05/recover-write-accessor-field-types/{baseline,fixed}/`
(pre-change: 8/9 refused and the rendered source set does **not** compile; post-change: 9/9
recovered and it compiles with `javac --release 8` exit 0 and prints the same line).

## st/ — the static-field negative (real javac 8)

`ST.java` is a nested class writing two **static** private fields (`int sc`, `String sr`). Real
javac 8 emits for that form `static int access$002(int)` — `iload_0; dup; putstatic; ireturn`
(no receiver parameter, `putstatic`, one-slot `dup`) — and `static String access$102(String)`
likewise. Neither matches the value-returning **instance-field** write accessor body this change
generalizes (the structural criteria `has_receiver == false`, `Field{is_static: false}` and the
`(L{owner};V)V` parameter shape refuse it), so both stay refused loudly and the class still
compiles. This family freezes that boundary on the real compiler's own output, not on a patch.

Behavior baseline: `7|q`.

## probes/ — closed-set and structure-break negatives (equal-length byte patches of `wa/WA.class`)

Real javac 8 cannot spell an out-of-table field descriptor (every Java field type is in the closed
set) and never writes a broken accessor body, so those boundaries are frozen with **equal-length
patches** of the frozen `WA.class`, the technique the `recover-javac8-getclass-null-check-idiom`
and allocation-qualifier fixtures use. Each patch is placed by the class-file structure walk
printed in `openspec/evidence/java-syntax-2026-10-05/recover-write-accessor-field-types/patch-provenance.txt`
(pool end 1433; `access$002` method entry flags at 1765, Code attribute at 1775, code bytes 1787
`2a1b5ab50009ac`), and every needle is asserted unique by that same walk.

| file | patch | what it refuses |
| --- | --- | --- |
| `WA-mismatch.class` | pool UTF8 `(LWA;Z)Z` → `(LWA;Z)I` (offset 423, 1 hit) | parameter and return no longer spell one type: the wrapper's own descriptor form |
| `WA-void.class` | pool UTF8 `(LWA;Z)Z` → `(LWA;V)V` (same entry) | `V` is outside the closed type set |
| `WA-dupbreak.class` | byte 1789 `0x5a` (`dup_x1`) → `0x59` (`dup`) | the copy shape of the body |
| `WA-nullrecv.class` | byte 1787 `0x2a` (`aload_0`) → `0x01` (`aconst_null`) | the receiver load of the body (frame-valid, so it reaches the shape test) |
| `WA-wrongowner.class` | `putfield`'s fieldref #9 class index → `java/lang/Object` | the field belonging to the accessor's own class |
| `WA-unsynth.class` | `access$002` method flags `0x1008` → `0x0008` (offset 1765) | `ACC_SYNTHETIC` stripped (the member is no longer the compiler's helper) |
| `WA-bcishift.class` | one `nop` (0x00) inserted at the head of the body (code_length 7 → 8, Code length 31 → 32, the member's `LineNumberTable` start BCIs each +1) | the `[0,1,2,3,6]` BCI layout, with every opcode of the table unchanged |
| `WA-handler.class` | `access$002` Code: exception_table_length 0 → 1 (one catch-all over the whole body), attribute length 31 → 39 | the no-exception-table criterion |

Every one of these keeps its loud refusal after the generalization, and the other eight accessors
of the same class stay recovered (the refusal is per member, never a whole-class collapse).

## wb/ — control positive with different identifiers and the array arm (source only)

`WB.java` is the identifier-generalization control the handoff requires: the fields are `tally`/
`bag`/`grid`, the writer class is `Writer`, and the three accessors write an `int`, a
`java.util.List` (`L…;` arm) and an `int[]` (`[…` arm). It is compiled in-test by
`javac --release 8`; real javac 8 was measured to produce the same three accessor shapes. Behavior
baseline: `11|0|2`.

## SHA256 of the frozen files

```text
6dee5a3f70294e807e73068bd2a56752362085dbbefd1654fbd003af4f28ee0d  probes/WA-dupbreak.class
7671af276fb937a064f1306ae3d05ca6010feb7b5473c8d708037f896868d88b  probes/WA-handler.class
a7a683477b03c23be7edc3741dcf7f267edc2737fe03c24980a1e39f21a23442  probes/WA-mismatch.class
7f804a51b5f40ef02d49a484a0a8d5b72e32dc7d99d353b75456f75ff94a07ac  probes/WA-nullrecv.class
b2100ba7506fc6ba4c3a4d9899346f92088f807f02da32a3510f7a0bb9fc6b0d  probes/WA-unsynth.class
59831d65d198a9ab2f5aad6ffff13dddd2f9b8eb2e7d5d0478371dd01dc01f41  probes/WA-void.class
cf3affba9e8337b09c608fc7a785efe6a1d94e66eda36f2bd1e542fe8955a775  probes/WA-wrongowner.class
f44a1520ef31df78aeabd305a4bf66ee59d7a5b3804eb311697998c3f644962d  st/ST.class
f0eeb389d141bae400d25277f929b3d33b2cf9ebb45b897d9562b719a18fe59e  st/ST$S.class
00576b2556f6491648dd86c1a1c25a4e9a4142217f041d5222202a9a3789cf7a  wa/WA.class
f981b690710db5dbb5a39b50a9ec35180e43f59f3d053bac40e4300c4d2ec1a4  wa/WA$S.class
335e19c4e3f1fd22ce3a3f478c1bdbaa85196200777b5cfb1c300a5c49811806  wa/WA.java
1a835750f840e72f35cfbe858a55c5eed8b7459a2683cfebc9ee01ed18f4f10a  st/ST.java
8f937d96d4236d60009ac72fe374743395f6f7c49520bbf12668ac0ddce3f5e3  wb/WB.java
```
