# P3 task 6.7 fixture: `ACC_VARARGS` writes the last parameter as `T...`

`v8/Var.class` is a real compiled sample: the sibling `Var.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Var.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Var` |
| class-file version | 52.0 (Java 8) |
| bytes | 255 |
| SHA-256 | `90f76ef375da8681b17a5de27d51c2100391e989176d11cca46f856632eaf673` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `arg1`) |
| read by | `tests/p3_varargs.rs` (all three declarations, and the one member that has to keep `int[]`) |

## What each member is for

`javac` states a variable-arity parameter **twice**, and neither statement is the declaration on its
own: the descriptor keeps the array the parameter really is (`(I[I)I`), and the member's own
`access_flags` carry `ACC_VARARGS` (`0x0080`, beside `ACC_STATIC`). A presentation that reads only
the descriptor writes `int[] arg1`; one that reads only the flag has no array to put the dots on. The
sample holds one member for each side of that boundary, so the flag's reach is pinned from both ends.

| member | descriptor | flags | before | after |
| --- | --- | --- | --- | --- |
| `many` | `(I[I)I` | `0x0088` (`ACC_STATIC \| ACC_VARARGS`) | `static int many(int arg0, int[] arg1)` | `static int many(int arg0, int... arg1)` |
| `merge` | `([[B)[B` | `0x0088` (`ACC_STATIC \| ACC_VARARGS`) | `static byte[] merge(byte[][] arg0)` | `static byte[] merge(byte[]... arg0)` |
| `plain` | `([I)I` | `0x0008` (`ACC_STATIC`) | `static int plain(int[] arg0)` | unchanged: `static int plain(int[] arg0)` |

`many` is the rule's plain face: the descriptor's `[I` is written `int...`, and the parameter is
never written `int[]` again. `merge` is its boundary: the dots take the place of the **last** `[]` of
the two the descriptor states, so the element type keeps the `[]` it has and the parameter is
`byte[]...` — the element type, then `[]`, then the dots — never `byte[][]` and never `byte...`.
`plain` is the control: an array parameter of a member whose own flags state no `ACC_VARARGS` keeps
the brackets the descriptor gives it, which is also what makes the dots visibly the flag's doing and
not the array's.

The flag is a declaration fact and nothing else. `many`'s body returns its first parameter and never
touches the array; `merge`'s body reads an array element (`parts[0]`, written `return arg0[0];`) and
`plain`'s reads an array length (`xs.length`, written `return arg0.length;`). The last two are the
point of the control pair: the body reads the parameter as the array the descriptor states, and the
declaration beside it writes the dots — the flag changes the spelling of the list, not the array the
slot holds. (Array reads are their own slice of the work; with them landed both bodies are written,
and where they are not the member keeps its declaration and quotes the artifact's own envelope.)
The declarations are what this fixture pins.

`Var.java` is committed exactly as it was compiled; the compile command is the one above, and the
`javac` warnings it prints are the ones every `--release 8` sample in this repository prints.
