# P3 fixture: an instance field's `++`, post and pre

`v8/Bump.class` is a real compiled sample: the sibling `Bump.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Bump.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Bump` |
| class-file version | 52.0 (Java 8) |
| bytes | 240 |
| SHA-256 | `7f2c077ca27ead7d3ed1a02947b8cc302e435ba2b6a0ca2040bc64636003038d` |
| debug attributes | none (`-g:none`): the receiver is the slot 0 of an instance method, so the name table states it as `this`, and the methods have no other local to name |
| read by | `tests/p3_field_increment.rs` (both texts, and the segment anchors of both shapes) |

## What the defect is, and what each member is for

`n++` and `++n` on one field are eight instructions each, and the two differ only in where the
`dup_x1` sits:

```text
int post();          int pre();
 0: aload_0            0: aload_0
 1: dup                1: dup
 2: getfield n         2: getfield n
 5: dup_x1             5: iconst_1
 6: iconst_1           6: iadd
 7: iadd               7: dup_x1
 8: putfield n         8: putfield n
11: ireturn           11: ireturn
```

The receiver's `dup` leaves one copy of `this` under the field's value for the `putfield`, and the
`dup_x1` is what makes the method return the update's own value: in `post` it copies the field's
**old** value before the sum (the `ireturn` reads that copy), in `pre` it copies the **sum** (the
`ireturn` reads that). The two were presented as nothing at all: the receiver's `dup` was quoted
(`belongs to no shape this run verified`), the `dup_x1` was `not part of the provable subset`, and
both the `putfield` and the `ireturn` quoted values "from an Other at BCI 5" (respectively 7) — so
neither the write the body performs nor the value it returns was written anywhere.

The shape the change reads is exactly those eight consecutive instructions, with the copy, the
constant `1` and the `iadd` in one of the two orders. Which update it is, is an identity of values
and not the order alone: the copy duplicated what the `getfield` read (a post-increment) or what
the `iadd` produced (a pre-increment). The rest of the evidence is what keeps the text honest: the
receiver's `dup` copies the receiver **load** and that same copy is the receiver the `getfield`
reads and the value under the one the `dup_x1` duplicates, the `getfield` and the `putfield` were
claimed by `field@1` for one member, the constant is `1`, and the `putfield` stores the update's own
result. A receiver that is not a load (a call's result) states no name this layer could write, so
its copy stays quoted as before.

| member | bytecode | pre-fix text | post-fix text |
| --- | --- | --- | --- |
| `post()I` | `aload_0; dup; getfield n; dup_x1; iconst_1; iadd; putfield n; ireturn` | explanation only: BCI 1 quoted (`belongs to no shape this run verified`), BCI 5 `not part of the provable subset`, and BCIs 8 and 11 quoted as values "from an Other at BCI 5" | `return this.n++;` |
| `pre()I` | `aload_0; dup; getfield n; iconst_1; iadd; dup_x1; putfield n; ireturn` | the same, with BCI 7 in place of BCI 5 | `return ++this.n;` |

Neither text writes `this.n = …`, and neither invents a local to carry the old value: the update is
one statement, because the value the method returns is the update's own. `pre` does not read the
field again for its `return`, and `post` does not turn into `++this.n` — the two are two programs,
and the copy's own operand is what tells them apart.

`<init>()V` is the third member: the constructor javac emits, so the class is one an instance
field's methods can be read from.
