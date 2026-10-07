# `recover-short-circuit-local-branch-reads` fixtures

The **branch condition position** of a proved short-circuit boolean local, on **both** compiler
legs. Each class was compiled from the same source by javac 23.0.1 with `--release 8` and by real
javac 8 (Corretto 1.8.0_432); both legs emit the identical instruction sequences (only
constant-pool indices differ), which is what makes the change's criterion a *bytecode* criterion
and not a compiler's habit.

| leg | directory | command |
| --- | --- | --- |
| javac 23.0.1 | `v8/` | `javac --release 8 -g -nowarn -d v8 BranchReads.java BranchReadNegatives.java` |
| Corretto 1.8.0_432 | `v8-javac8/` | `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -g -nowarn -d v8-javac8 BranchReads.java BranchReadNegatives.java` |

Debug information is kept (`-g`) so every local the presentation names is the source's own name.
The change's main anchor is not here: it is the operator-remainder patrol's frozen `OP2`
(`openspec/evidence/java-syntax-2026-10-05/operator-remainder-patrol/fixture/op2.jar`, SHA-256
`07eee287d7412bbb9c0292ca26faa66d0a4bccd10f75c7818f88eb325a9b5197`), compiled by the patrol with
javac's default (no `LocalVariableTable`, so its local is `local1`).

## What each member is for, and the bytes that make it

Every positive member is the same chain followed by a different condition position:

```text
    0: iinc   0, 1        // (x += 1)
    3: iload_0
    4: ifle   15          // (x += 1) > 0  -> short-circuit
    7: iload_0
    8: ifle   15          // x > 0
   11: iconst_1
   12: goto   16
   15: iconst_0
   16: istore_1           // boolean b = <the chain>
   17: iload_1            // <- the read this change admits
   18: ifeq    25         // <- the branch: its consumer, in the store's own canonical block
```

`BranchReads` — the positions the arm admits:

* `ternaryRead` — `return b ? x : -1;`: the branch's two arms (`iload_0`, `iconst_m1`) merge at
  BCI 26 and are presented by the existing conditional-value channels;
* `ifStatement` — `if (b) { return x; } else { return -1; }`: the statement's own two returns;
* `midChain` — `return (x > 0) && b && (x < 100);`: the stored local is the **middle** test of a
  second chain. The arm reaches it, and the second chain's region composition spells the tail
  nested (`x > 0 && (b && x < 100)`), which evaluates in the source's order;
* `main` — a plain `println` sequence (no concatenation chain), so the whole class recovers and the
  replay can compile and run it as it stands.

`BranchReadNegatives` — the boundaries this change does **not** cross:

* `loopCondition` — `while (b) { … }`: the loop's header is a canonical block of its own, so the
  read's region path (`[1]`) is not the declaration's (`[0]`) and the *same gate's* cross-region
  criterion refuses the local before the consumer whitelist is reached. The loop itself is still
  presented (`while (b != 0)`, the local typed `int`) beside the quoted chain;
* `crossCatch` — the chain stored inside a `catch` and read outside the `try`: refused one layer
  earlier, at region ownership (`canonical block at BCI 18 … has more than one owner`), before the
  gate is ever asked.

Both boundaries are stated by the presentation, and neither compiles: the loop's quoted chain
leaves `int b;` uninitialized, and the crossing form is quoted whole. That is the sound direction —
a method a reader cannot compile, never one that compiles and behaves differently.

## The two hand-patched controls

`controls/NumericBranch.class` and `controls/NotZeroBranch.class` are `v8/BranchReads.class` with
**one opcode byte** changed: the `ifeq` at BCI 18 of `ifStatement` (file offset 709 in both legs,
found by the eight-byte pattern `3c 1b 99 00 05 1a ac 02 ac`) patched to `iflt` (0x9b) and to
`ifne` (0x9a). Each file keeps the class name `BranchReads`, so it is read under the name it
states; nothing else differs, and both verify (`java -Xverify:all`).

* `NumericBranch` — a numeric test on the loaded value is **not** a Boolean consumer: `ifStatement`
  keeps the whole refusal (`the short-circuit chain from BCI 4 through 8 reaches a shared value
  consumer at BCI 16`), byte-identical to the baseline. This is the arm's width control.
* `NotZeroBranch` — the inverted zero-test *is* admitted, and the presentation spells the sense it
  actually has: `if (!b) { return x; } else { return -1; }`. This is the arm's second opcode.

## SHA-256

| file | SHA-256 |
| --- | --- |
| `BranchReads.java` | `c6fd5e3a0b052bfa7b12d0333d1463608c97aff3650547abd13279b0a3e21bff` |
| `BranchReadNegatives.java` | `124321baa87b3e695b86195cd12421513d2feb297a91fd77acd2378b826c8c3f` |
| `v8/BranchReads.class` | `f0279a02d8ebd4523ca2e768568e8b5a4e108df554de1ff09e7edc0230fe1a70` |
| `v8/BranchReadNegatives.class` | `693461fe9e459cdc0ecd474b53f8529558c46d4f4dd79f71cc4f1fdd68710f20` |
| `v8-javac8/BranchReads.class` | `6b01b7943210874435fe2dbb01d0d21b56446e2f0d63f67861deefae54e10be4` |
| `v8-javac8/BranchReadNegatives.class` | `9a7d9e8b66945abeba85013e3ac3da1dbe8054ae69230851e36676c5a17486d2` |
| `controls/NumericBranch.class` | `59b9b11c26b26ec8992ecdbc8e0551ac0a196f2091340746cf73deca1437a985` |
| `controls/NotZeroBranch.class` | `a85907bfe6de8de8a3e7631108f00f7adc0f321380a03c6042a5e6991c14f9f0` |

## The frozen answers

Measured by running the committed classes (JDK 23.0.1, `java -Xverify:all`):

```text
BranchReads         1 / -1 / 1 / -1 / true / false
BranchReadNegatives 3 / 0 / 1
OP2                 12/-4/2147483644/16/NaN/Infinity/-Infinity/true/true/1
```

`tests/recover_short_circuit_local_branch_reads.rs` pins them: the default suite over the
presentations (both legs), and the `#[ignore]`d replay that strips the presentations the way the
patrols' own stripped sources were made, compiles each with both javac legs and runs both under
`java -Xverify:all`.

The replay's one substitution is stated rather than hidden: `OP2`'s own `main` is a **registered
residual** of the concatenation chain's saved-producer family — refused before and after this
change — so its `jarde_refused_body();` marker is replaced by the frozen `OP2.java`'s own `main`
body (asserted to be the source's, not a transcription) before compiling. Everything else in the
class is the recovered text, and the line the replay compares is the recovered methods' answers,
`condAssignOld(0)` included.

## Reproducing

```text
cd tests/fixtures/recover-short-circuit-local-branch-reads
javac --release 8 -g -nowarn -d v8 BranchReads.java BranchReadNegatives.java
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -g -nowarn -d v8-javac8 BranchReads.java BranchReadNegatives.java
python3 - <<'PY'
import pathlib
src = pathlib.Path('v8/BranchReads.class').read_bytes()
pat = bytes.fromhex('3c1b9900051aac02ac')
at = src.index(pat) + 2
for name, opcode in (('NumericBranch', 0x9b), ('NotZeroBranch', 0x9a)):
    data = bytearray(src)
    data[at] = opcode
    pathlib.Path(f'controls/{name}.class').write_bytes(bytes(data))
PY
```
