# P3 task 2c.19 fixture: `append(C)` is a character operand of a string `+`

`v8/Letters.class` is a real compiled sample: the sibling `Letters.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Letters.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Letters` |
| class-file version | 52.0 (Java 8) |
| bytes | 453 |
| SHA-256 | `c42450c842ef4e51889c48e28d09696ca56acf3f79b2b39d03969825684e5240` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_concat_char.rs` (the exact text of both members and the `char` parameter of each `append`) |

## What the member is for

`append` is overloaded, and `concat@1` presents a verified chain as one `+` expression. `append(char)`
writes one character, which `+` writes as well — but only from a **string context**, since `+` on two
int-shaped operands adds numbers. The emitter already starts the expression in that context whenever
its first part is not a `String` (`"" +`), so the character is concatenated and the overload is one
`+` reproduces. Before this task the rule refused the whole chain on the parameter type alone, so
both members were quoted as bytecode; `char[]` and `CharSequence` still are.

| member | source | bytecode | text |
| --- | --- | --- | --- |
| `letter(char)` | `"x" + c` | `append(String)` (`ldc "x"`), `append(C)` | `return "x" + arg0;` |
| `only(char)` | `"" + c` | `append(String)` (`ldc ""`), `append(C)` | `return "" + arg0;` |

`javac --release 8` lowers `"" + c` to a chain **with** the empty string as a part of its own
(`append("")` and then `append(C)`), so `only` is not the decoration the emitter writes for a first
part that needs conversion: the emitter adds nothing here — the `""` part is real bytecode. The text
keeps it, and the character is never written as the bare `arg0` a lost conversion would leave.

## The bytecode

```text
public Letters();  // 2a b7 00 01 b1
     0: aload_0
     1: invokespecial #1                  // Method java/lang/Object."<init>":()V
     4: return

static java.lang.String letter(char);  // bb 00 07 59 b7 00 09 12 0a b6 00 0c 1a b6 00 10 b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: ldc           #10                 // String x
     9: invokevirtual #12                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    12: iload_0
    13: invokevirtual #16                 // Method java/lang/StringBuilder.append:(C)Ljava/lang/StringBuilder;
    16: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    19: areturn

static java.lang.String only(char);  // bb 00 07 59 b7 00 09 12 17 b6 00 0c 1a b6 00 10 b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: ldc           #23                 // String
     9: invokevirtual #12                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    12: iload_0
    13: invokevirtual #16                 // Method java/lang/StringBuilder.append:(C)Ljava/lang/StringBuilder;
    16: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    19: areturn
```

## Reproducing

```text
cd tests/fixtures/p3-concat-char
mkdir -p v8
javac --release 8 -g:none -d v8 Letters.java
shasum -a 256 v8/Letters.class   # c42450c842ef4e51889c48e28d09696ca56acf3f79b2b39d03969825684e5240
```
