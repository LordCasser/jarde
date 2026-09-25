# P3 task 6.4 fixture: one NUL inside a string constant (Modified UTF-8 `C0 80`)

`v8/Controls.class` is a real compiled sample: the sibling `Controls.java` compiled by **javac
23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Controls.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Controls` |
| class-file version | 52.0 (Java 8) |
| bytes | 196 |
| SHA-256 | `7ecf16914cef862ef21d534f7c4431022171a3d8942154953772f142654f8d2f` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_mutf8.rs` (the recovered text of `controls`) |

## What the member is for

`controls()` returns the string constant `"a\u0000b"`, whose constant-pool bytes are `61 C0 80 62`:
a class file writes U+0000 in the **two-byte** form Modified UTF-8 gives it (JVMS 4.4.7), never as a
raw zero byte. That form is the overlong UTF-8 spelling of U+0000, so a reader that decodes the
`Utf8` entry as standard UTF-8 refuses it and makes **two** replacement characters of the one NUL —
`a\ufffd\ufffdb` for text whose class file states `a\u0000b`. The fixture is the smallest class that
states the difference: one `ldc` of one string constant.

| member | bytecode | bytes of `#8` | pre-fix text | post-fix text |
| --- | --- | --- | --- | --- |
| `controls()` | `ldc #7; areturn` (`12 07 b0`) | `61 C0 80 62` | `return "a\ufffd\ufffdb";` — two replacements where the class file states one NUL | `return "a\u0000b";` — the NUL `escape_string` writes as `\u0000` |

The constant pool entry is `#8 = Utf8 a\u0000b` (tag `01`, length `00 04`, then `61 C0 80 62`), and
`#7 = String #8` is the one `ldc` names. `javap -v -p` prints that entry as `a\u0000b`, which is the
decoding this fixture exists to pin; the raw bytes are what settle it.

## The bytecode

```text
public Controls();  // 2a b7 00 01 b1
     0: aload_0
     1: invokespecial #1                  // Method java/lang/Object."<init>":()V
     4: return

static java.lang.String controls();  // 12 07 b0
     0: ldc           #7                  // String a\u0000b
     2: areturn
```

## Reproducing

```text
cd tests/fixtures/p3-mutf8
mkdir -p v8
javac --release 8 -g:none -d v8 Controls.java
shasum -a 256 v8/Controls.class   # 7ecf16914cef862ef21d534f7c4431022171a3d8942154953772f142654f8d2f
javap -v -p v8/Controls.class     # #8 = Utf8  a\u0000b
```
