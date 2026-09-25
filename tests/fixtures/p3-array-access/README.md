# P3 fixture: what an array is written as — its creation, its subscripts, its length, its element

`v8/Grid.class` is a real compiled sample: the sibling `Grid.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Grid.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时…` / `目标值 8 已过时` /
`要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and writes the class
anyway, exit code 0. The compiler is a generation-only input: it is not needed at run time, so the
sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Grid` (default package) |
| class-file version | 52.0 (Java 8) |
| bytes | 1184 |
| SHA-256 | `f0e1c73b84c4968698ad200759711d0952c4ed8133b1065e0369abbdf985172f` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `local1`) |
| read by | `tests/p3_array_access.rs` (every member's text, and the one member still quoted) |

## What each member is for

The sample is one member per array rule of the P3 2b slice (P10). The dispatch table a compiler
builds for an enum `switch` is deliberately **not** here: that read is `enumswitch@1`'s claim and is
pinned in `crates/jarde-java/tests/p3_patterns.rs`. The initializer chain `new T[]{…}` (2b.7) and the
`arraylength` a **loop test** reads (2b.6) are the batch's two boundaries, and `literal` is one of
them measured rather than assumed.

| member | bytecode (`javap -c -p`) | text this slice writes |
| --- | --- | --- |
| `created(II)I` | `iload_0; newarray int; astore_2; aload_2; iconst_0; iconst_1; iastore; aload_2; iload_1; iaload; ireturn` | `int[] local2 = new int[arg0]; local2[0] = 1; return local2[arg1];` |
| `pick([II)I` | `aload_0; iload_1; iaload; ireturn` | `return arg0[arg1];` |
| `at([[III)I` | `aload_0; iload_1; aaload; iload_2; iaload; ireturn` | `return arg0[arg1][arg2];` |
| `flag()Z` | `iconst_3; newarray boolean; astore_0; aload_0; iconst_0; baload; ireturn` | `boolean[] local0 = new boolean[3]; return local0[0];` |
| `test([ZI)Z` | `aload_0; iload_1; baload; ireturn` | `return arg0[arg1];` |
| `letter([CI)C` | `aload_0; iload_1; caload; ireturn` | `return arg0[arg1];` |
| `code([CI)I` | the same body as `letter` | `return arg0[arg1];` |
| `first([BI)I` | `aload_0; iload_1; baload; ireturn` | `return arg0[arg1];` |
| `wide([JI)J` | `aload_0; iload_1; laload; lreturn` | `return arg0[arg1];` |
| `real([FI)F` | `aload_0; iload_1; faload; freturn` | `return arg0[arg1];` |
| `precise([DI)D` | `aload_0; iload_1; daload; dreturn` | `return arg0[arg1];` |
| `small([SI)S` | `aload_0; iload_1; saload; ireturn` | `return arg0[arg1];` |
| `size([I)I` | `aload_0; arraylength; ireturn` | `return arg0.length;` |
| `put([III)I` | `aload_0; iload_1; iload_2; iastore; aload_0; iload_1; iaload; ireturn` | `arg0[arg1] = arg2; return arg0[arg1];` |
| `setByte([BIB)V` | `aload_0; iload_1; iload_2; bastore; return` | `arg0[arg1] = arg2;` |
| `strings(I)[Ljava/lang/String;` | `iload_0; anewarray java/lang/String; areturn` | `return new java.lang.String[arg0];` |
| `box(Ljava/lang/String;)[Ljava/lang/String;` | `iconst_1; anewarray java/lang/String; astore_1; aload_1; iconst_0; aload_0; aastore; aload_1; areturn` | `java.lang.String[] local1 = new java.lang.String[1]; local1[0] = arg0; return local1;` |
| `cell(II)I` | `iload_0; iload_1; multianewarray [[I, 2; astore_2; aload_2; iconst_0; aaload; iconst_0; bipush 7; iastore; aload_2; iconst_0; aaload; iconst_0; iaload; ireturn` | `int[][] local2 = new int[arg0][arg1]; local2[0][0] = 7; return local2[0][0];` |
| `literal()[I` | `iconst_3; newarray int; dup; iconst_0; iconst_1; iastore; dup; iconst_1; iconst_2; iastore; dup; iconst_2; iconst_3; iastore; areturn` | **quoted** (2b.7's chain: the creation's value is read by a `dup`, and no rule of this slice presents that shape) — the quote names BCI 1 and says the creation's value is read by nothing this body writes |
| `chained(I)[I` | `iconst_1; newarray int; dup; iconst_0; iload_0; iastore; astore_1; aload_1; areturn` | **quoted** (2b.7's chain again, this time stored into a local) — the store is quoted at BCI 7, so no statement declares `local1`, and the `return` that read it is quoted at BCIs 8/9 instead of spelling a name nothing declares (P3 2b.2) |

## The rules the members pin

* **P3 2b.1/2b.4 — one subscript, with the element type the *array* states.** `pick`, `code`, `put`
  and the rest are the plain reads and writes: `array[index]` and `array[index] = value;`. `first`
  (`[B`), `code` (`[C`) and `small` (`[S`) write no conversion the bytecode did not perform, and
  `flag` is a `boolean` read **only** because its array is proven `[Z` — `first` is read with the very
  same `baload` opcode and is not a boolean.
* **P3 2b.2 — a written local is declared.** `created`, `box` and `cell` hold the locals this rule is
  about, and the test checks the text's own names: no member spells a `localN` it never declares. The
  sample's `-g:none` is what makes the question real — every slot is named by its ordinal. `chained`
  is the negative: the write that would have declared its local is quoted, so the use that read the
  slot is quoted as well instead of being written as a name with no declaration behind it.
* **P3 2b.3 — one creation per instruction.** `cell` is `new int[arg0][arg1]`, written once, from the
  pool class `[[I` and the operand's dimension count; it is not two one-dimensional creations.
* **P3 2b.5 — the pool class is the element.** `strings` and `box` are `new java.lang.String[n]`, and
  the local `box` declares is `local1` — the slot the store really wrote, in a static member whose
  slot 0 is its parameter.
* **the boundary the batch leaves.** `literal` is quoted because its creation's value reaches a
  `dup` (the initializer chain of 2b.7) and because the loop-test `arraylength` of 2b.6 is not
  presented by this slice either. Both are stated as gaps rather than half written.
