# P3 fixture: a `char` selector's `switch` keys are character literals

`v8/Letters.class` is a real compiled sample: the sibling `Letters.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Letters.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Letters` |
| class-file version | 52.0 (Java 8) |
| bytes | 301 |
| SHA-256 | `913185e82f7a95479c09bbdcb05dab9edb6de263c2bfc519d433884f90cad5b5` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `local1`) |
| read by | `tests/p3_char_switch.rs` (both member texts) |

## What each member is for

`switch (c) { case 'a': … }` over a `char` and `switch (n) { case 97: … }` over an `int` are
different programs that compile the same key: a `lookupswitch` payload states `97`, and whether the
source of a `char` switch wrote the character or the number is not in the class file at all — `case
97:` over a `char` is legal and means the same character. What the class file does state is the
selector's own type, and the presentation writes the key as that type: a selector whose text already
presents `char` writes the character literal, every other selector keeps the decimal number.

| member | bytecode | text |
| --- | --- | --- |
| `letter(C)I` | `iload_0; lookupswitch {97 → 28, 98 → 30, default → 32}; iconst_1; ireturn; …` | `switch (arg0) { case 'a': return 1; case 'b': return 2; default: return 0; }` |
| `number(I)I` | `iload_0; lookupswitch {97 → 20, default → 22}; iconst_1; ireturn; …` | `switch (arg0) { case 97: return 1; default: return 0; }` |

The two members are the rule and its control:

* **`letter`** is the shape the change is about. Its parameter descriptor states `C`, which is the
  one fact that tells a `char` from an `int` (the frames state one slot shape for both), and the
  keys it selects on are written `case 'a':` and `case 'b':` — no key of this member is left as
  `case 97:`.
* **`number`** is the control in the other direction: the bytecode is the same `lookupswitch` with
  the same key `97`, and the only difference is the selector's presented type (`int`), so its key
  stays `case 97:`. A rule that read the *value* 97 instead of the type would write `case 'a':`
  here, which is a program the class file does not state.

A key outside `0..=0xFFFF` stays decimal even over a `char` selector: such a key is not any
character's value, so no character literal can state it. `javac` accepts no such `case` label over a
`char`, so that boundary is pinned by the emitter's own unit test in `crates/jarde-java/src/emit.rs`
rather than by a class this fixture could compile — the case a hand-built class file can still state.
