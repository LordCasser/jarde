# P3 fixture: the instance a construction leaves behind

`v8/Built.class` is a real compiled sample: the sibling `Built.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Built.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Built` |
| class-file version | 52.0 (Java 8) |
| bytes | 606 |
| SHA-256 | `0a6570581fe355a3b5b098a792f39afbc63f335c21ebe81dd3f4669b3af864e1` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg1`) |
| read by | `tests/p3_new_value.rs` (all seven member texts, and the patched boundary case) |

## What the defect is, and what each member is for

`javac --release 8` builds a construction as one run of instructions — `new T; dup; args…;
invokespecial T.<init>` — and its two copies are the constructor's receiver and the constructed
value. `new@1` already verified that chain and already wrote the `new` expression where a **store**
consumes the leftover. Every other single consumer was refused, and the refusal was in the site plan,
not in the text: the plan asks whether the instance is written into its consumer's own text, and it
did not count a field access — the instruction that reads the value a `putfield`/`putstatic` stores —
as a place the value is written (P3 2c.26). The same plan already counted a call, a `return` and a
test, which is why 2c.27's positions are presented and pinned here as controls.

| member | bytecode | pre-fix text | post-fix text |
| --- | --- | --- | --- |
| `<init>()V` | `aload_0; invokespecial Object.<init>; aload_0; new; dup; invokespecial Object.<init>; putfield a; return` | `super();` and the construction quoted (`@bytecode 5`, `8`, `12 9`) | `super();` then `this.a = new java.lang.Object();` |
| `set()V` | `aload_0; new; dup; invokespecial Object.<init>; putfield a; return` | quoted at `@bytecode 1`, `4`, `8 5` | `this.a = new java.lang.Object();` |
| `setStatic()V` | `new; dup; invokespecial Object.<init>; putstatic s; return` | quoted at `@bytecode 0`, `3`, `7 4` | `Built.s = new java.lang.Object();` |
| `localNew()` | `new; dup; invokespecial Object.<init>; astore_1; aload_1; areturn` | `java.lang.Object local1 = new java.lang.Object(); return local1;` | unchanged: the store consumer's spelling is the one this layer always wrote |
| `directNew()` | `new; dup; invokespecial Object.<init>; areturn` | `return new java.lang.Object();` | unchanged: the `return` of 2c.27 was already presented |
| `take(Ljava/lang/Object;)` | `aload_1; invokestatic String.valueOf; areturn` | `return java.lang.String.valueOf(arg1);` | unchanged: the callee, so the argument's own text is read against a member that does nothing |
| `argNew()` | `aload_0; new; dup; invokespecial Object.<init>; invokevirtual take; areturn` | `return this.take(new java.lang.Object());` | unchanged: the call-argument position of 2c.27 |

The three field writes are the rule and the rest are the controls:

* **`<init>`/`set`/`setStatic`** are the shape 2c.26 is about. The instance is written **where the
  write runs**, so the constructor's initializer order — `super()` first, then the field — is the
  bytecode's order, and no local is invented to hold the instance. An instance write takes the
  receiver `field@1` already writes (`this`), a static write its owner type (`Built.s`), and the
  construction call is never written a second time as a statement of its own: one `new` expression in
  one place is the whole of it.
* **`localNew`** is the control that keeps the store consumer exact: `new@1`'s existing spelling
  (`local1 = new java.lang.Object(); return local1;`) is what it was, name and all.
* **`directNew`/`argNew`/`take`** are 2c.27's positions — a `return` and a call argument — which were
  already presented and are pinned here so neither this change nor a later one moves them: the `new`
  is the returned value and the argument, nested in the call the way every other argument expression
  is, with no local and no second statement.

The companion case in `tests/p3_new_value.rs` patches this same class's `localNew` body — a
same-width `astore_1; aload_1` → `dup; pop` — so the leftover is read by **two** instructions. That
body is deliberately refused: a construction is one `new` expression in one place, and two consumers
of one instance have no single Java spelling. The patch adds no class and no fixture: the sample the
census counts is the one class above.
