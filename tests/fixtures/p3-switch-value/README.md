# P3 fixture: a `switch` expression is one `return` per arm

`v21/Joined.class` is a real compiled sample: the sibling `Joined.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 21 -g:none -d v21 Joined.java
```

`--release 21` is the lowest release a `switch` expression compiles at (`--release 8` refuses the
arrow form), and this compiler prints nothing for it: no deprecation warning, exit code 0. The
compiler is a generation-only input: it is not needed at run time, so the sample is committed as
bytes.

| property | value |
| --- | --- |
| class | `Joined` |
| class-file version | 65.0 (Java 21) |
| bytes | 306 |
| SHA-256 | `6734f743f5913a773aaa0f413a68d5fb4b5d8f3244479cef2a5497c8c4f9cccd` |
| debug attributes | none (`-g:none`), so the arm's own local is named by its ordinal (`arg0`, `local1`) |
| read by | `tests/p3_switch_value.rs` (both member texts) |

## The bytes

```text
static int expr(int);
    Code:
       0: iload_0
       1: lookupswitch  { // 2
                     1: 28
                     2: 32
               default: 36
          }
      28: iconst_2
      29: goto          37
      32: iconst_3
      33: goto          37
      36: iconst_0
      37: ireturn

static int yielded(int);
    Code:
       0: iload_0
       1: lookupswitch  { // 1
                     1: 20
               default: 28
          }
      20: iload_0
      21: iconst_1
      22: iadd
      23: istore_1
      24: iload_1
      25: goto          29
      28: iconst_0
      29: ireturn
```

## What the defect is, and what each member is for

`javac` compiles both expressions to an ordinary `lookupswitch`: each arm pushes **one** value and
transfers to one shared `ireturn`, and the no-match arm usually falls into it rather than returning
on its own (`36: iconst_0` falls through to `37: ireturn`). The join block's entry state therefore
names a stack **phi** — the value every arm hands it — and `Builder::render_value` refuses every
value of that shape (`the value at BCI 37 is the entry state of stack depth 0, which no instruction
produced`). So each arm was written as an empty `break` and the join was quoted:

| member | pre-fix text |
| --- | --- |
| `expr(I)I` | `switch (arg0) { case 1: break; case 2: break; default: break; }` and `<blank>// @bytecode 37` / `the value at BCI 37 is the entry state of stack depth 0, which no instruction produced` |
| `yielded(I)I` | the same with `case 1: int local1 = arg0 + 1; break;` and the quote at BCI 29 |

The text that states the shape is one `return` inside each arm, written where that arm's value is
produced, with the join's own instruction skipped: `emit.rs` writes no `break` after a `return`
(an unreachable `break` is worse than none), and no `return switch` node is invented — a switch
expression is its arms' returns. The rule is all or nothing, and the two members pin both halves of
it:

| member | bytecode | post-fix text |
| --- | --- | --- |
| `expr(I)I` | `iload_0; lookupswitch {1 → 28, 2 → 32, default → 36}; 28: iconst_2; goto 37; 32: iconst_3; goto 37; 36: iconst_0; 37: ireturn` | `switch (arg0) { case 1: return 2; case 2: return 3; default: return 0; }` — three `return`s, no `break`, no `@bytecode` |
| `yielded(I)I` | `…; 20: iload_0; iconst_1; iadd; istore_1; iload_1; goto 29; 28: iconst_0; 29: ireturn` | `switch (arg0) { case 1: int local1 = arg0 + 1; return local1; default: return 0; }` |

`yielded` is the boundary the rule must not cross: the value the arm leaves is the **load** of
`local1`, not the `iadd` that computed it, so the statement the arm already wrote is kept and the
text says `return local1` — never `return arg0 + 1`, which is a program the arm's instructions do
not state (it would drop the store `int x = n + 1;` performed).

The join is only the shape when it holds exactly one instruction, a `return` of exactly one value,
every arm leaves exactly one value, and the arms are the **only** way into it. A join with a store
in front of its `return` (`int r = 0; switch (n) { case 1: r = 1; break; default: r = 2; } return
r;` holds `iload_1; ireturn`), an arm that leaves nothing, an arm that leaves several, and a join
another block also transfers to — a `switch` expression nested as one arm of another switch, or as
one arm of a `?:`, where the enclosing structure is what states that block — all keep today's text,
the join's own quoted statement included, because a `return` appended to some arms and not others,
or a skipped join for a value no arm was given the statement to return, would state a program the
bytes do not have.

The control in the other direction is `tests/fixtures/p3-char-switch/`: its arms already end in
their own `ireturn` (no shared join return exists), so nothing in either of its texts may gain a
second `return`.
