# P3 fixture: a branch whose successor **is** the join is a one-armed `if`

`v8/OneArmed.class` is a real compiled sample: the sibling `OneArmed.java` compiled by
**javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 OneArmed.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `OneArmed` |
| class-file version | 52.0 (Java 8) |
| bytes | 331 |
| SHA-256 | `963f64e154c48750111b1256d24cdb3b38a23bc92653562ea96a9c39331f014a` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `local1`) |
| read by | `tests/p3_one_armed_if.rs` (the three texts and the content plane) |

## What the defect is, and what each member is for

A two-successor branch has two shapes: the join is a block both arms **walk into**, or one successor
**is** the join — the arm the branch reaches there is empty, and the statement is an `if` with no
`else`. `javac` writes the second shape for every source `if` without an `else` whose arm is the
fall-through path. The region walk required both successors to reach the join as a *later* block, so
the second shape fell back as `ArmsDoNotMeet` with `next = None`: the prefix and the branch were
quoted under that reason, the arm's block and the block after the `if` were quoted as uncovered, and
no statement of the member was written.

| member | bytecode | pre-fix text | post-fix text |
| --- | --- | --- | --- |
| `oneArmed(I)I` | `iconst_1; istore_1; iload_0; ifle 8; iload_0; istore_1; iload_1; ireturn` | explanation only: "the arms of the branch in block 0 do not meet at one join" plus blocks 6 and 8 uncovered | `int local1; local1 = 1; if (arg0 > 0) { local1 = arg0; } return local1;` |
| `elseOnly(I)I` | `iconst_1; istore_1; iload_0; ifle 9; goto 11; iload_0; istore_1; iload_1; ireturn` | unchanged by the fix | `int local1; local1 = 1; if (arg0 > 0) { } else { local1 = arg0; } return local1;` |
| `bothArms(I)I` | `iload_0; ifle 9; iconst_2; istore_1; goto 11; iconst_3; istore_1; iload_1; ireturn` | unchanged by the fix | `int local1; if (arg0 > 0) { local1 = 2; } else { local1 = 3; } return local1;` |

The three members are the two directions the one-armed shape takes and the control beside them:

* **`oneArmed`** is the shape the change is about: the branch's **taken** successor is the block after
  the `if`, so the arm is the fall-through one and no `else` is written for the empty taken arm.
* **`elseOnly`** states the opposite arm of the same source idea — `if (n > 0) { } else { x = n; }`.
  Its **empty then** arm is a block of its own that jumps to the join, so neither successor *is* the
  join, the two-arm path recovers it, and the `else` stays: the assignment is in the arm the branch
  transfers to, and an arm that is empty because it *is* the join is the only one written without
  `else` — an arm with a block that reaches the join is not omitted.
* **`bothArms`** is the regression control: two arms that assign on both sides of the branch, neither
  of them the join, unchanged.
