# P3 fixture: the field read a local alias holds

`v8/Lazy.class` is a real compiled sample: the sibling `Lazy.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Lazy.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the classes anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes. `Lazy$Holder.class` is the nested class the same run
wrote; no test reads it, and the reader needs no class for the name `Lazy$Holder` — the name is all
`get`'s bytes state.

| property | value |
| --- | --- |
| class | `Lazy` (`v8/Lazy.class`) |
| class-file version | 52.0 (Java 8) |
| bytes | 445 |
| SHA-256 | `04d2281186747138e0037f19191b3326ec1901de97601f269466752a37c98959` |
| nested class written beside it | `Lazy$Holder.class`, 192 bytes, SHA-256 `89ab661a260eab02145bd88b9b56139d230de603bdf7c0c035a42ae48f05e9d3` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg1`, `local0`) |
| read by | `tests/p3_alias_field.rs` (the alias read and its two receiver controls) |

One field was added to the batch's own source, and it is not a change of the shape under test: the
control member `viaThis` reads `this.v`, and a class that declares no `v` does not compile, so the
sample declares `int v;` beside the static `h`. `get` is byte for byte the batch-96 body.

## What the defect is, and what each member is for

`get` caches a static field in a local, fills the local on the null arm and reads the field back
through the local after the merge:

```text
0: getstatic h          3: astore_0          4: aload_0          5: ifnonnull 20
8: new Lazy$Holder     11: dup              12: invokespecial Lazy$Holder.<init>
15: astore_0           16: aload_0          17: putstatic h
20: aload_0            21: getfield Lazy$Holder.v:I        24: ireturn
```

The class file states the slot's type at the merge point — `StackMapTable: append_frame,
offset_delta = 20, locals = [ class Lazy$Holder ]` — and the value the `getfield` reads its receiver
from is the slot's own (the merge point's entry phi of local 0, then the `aload_0` of it). `field@1`
read the receiver's type off the instruction that produced the value alone, so a value the **frames**
state and no instruction produces fell into the "this run does not state" refusal and both the read
and the `return` were quoted. The rule now reads the slot's frame entry at the use point for such a
receiver.

| member | bytecode | text before the rule | text after it |
| --- | --- | --- | --- |
| `get()I` | the body above | `Lazy$Holder local0; local0 = Lazy.h; if (local0 == null) { local0 = new Lazy$Holder(); Lazy.h = local0; }` then the field read **quoted** at BCI 21 and the `return` quoted at BCI 24 | the same body with `return local0.v;` and no quote |
| `viaParam(LLazy$Holder;)I` | `aload_1; getfield Lazy$Holder.v; ireturn` | `return arg1.v;` | unchanged — the control whose receiver value states its own class |
| `viaThis()I` | `aload_0; getfield Lazy.v; ireturn` | `return this.v;` | unchanged — the receiver the frames state as the class being read, which no rule requalifies |
| `<init>()V` | `aload_0; invokespecial Object.<init>; return` | `super(); return;` | unchanged: the constructor javac emits, so the class is one a class file can be read from |

## The rule the members pin

* **P3 2c.32 — the receiver is the frames' slot type where the value itself states none.** A field
  read whose receiver is a local alias is a receiver no instruction produced: the merge point's phi
  (or the load of the slot it fills) carries the *slot's* class, and the frames state it. The batch-96
  body is the shape — `local0 = Lazy.h; if (local0 == null) { local0 = new Lazy$Holder(); Lazy.h =
  local0; } return local0.v;` — and the acceptance is that the read is presented from the frames'
  own answer for the slot rather than refused. `viaParam` and `viaThis` are the controls: a receiver
  whose value already states its class (a reference parameter, `this`) takes the same path it always
  did, and neither may be requalified by the new one. Static accesses have no receiver at all and
  are untouched (`Lazy.h` and `Lazy.h = local0` in `get`'s own text).
* **Where the frames' answer comes from.** jarde's frames are the merged entry states of the
  canonical graph, not the class file's `StackMapTable` (no layer of this build decodes that
  attribute). The two writes that meet at BCI 20 spell the one class **two ways**: the `getstatic`
  carries the field descriptor's slice (`LLazy$Holder;`) and the `new`'s class entry carries the
  internal name (`Lazy$Holder`). The frame pass merges references by comparing the names the facts
  reached it with, so that meeting used to answer `Ref(Unknown)` — the class the merge point really
  has was dropped in the merge, and the field rule's refusal followed from it. Normalizing the two
  spellings in the merge (they are one type: `L…;` is a wrapping, not part of the identity) is what
  makes the frames state `Lazy$Holder` here, which is the fact the rule reads.
