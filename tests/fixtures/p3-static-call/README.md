# P3 fixture: a static call names the class its own pool entry names

`v8/Calls.class` is a real compiled sample: the sibling `Calls.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Calls.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Calls` |
| class-file version | 52.0 (Java 8) |
| bytes | 317 |
| SHA-256 | `0b729df9aa55863efe0330faab65ca3c171ac22f8a476df4766c45010ee3361d` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`) |
| read by | `tests/p3_static_call.rs` (both method texts, and the two names neither may carry) |

## What the defect is, and what each member is for

`invokestatic` reads no receiver from the stack, so the presented call was the member's **bare**
name: `Integer.valueOf(n)` was written `valueOf(arg0)`, a name this class declares nowhere. The
class the call reaches is not a guess — it is the owner of the instruction's own `Methodref` entry —
and the text has to name it unless it is the class the body already belongs to. `local(n)` calls
`own`, which this very class declares: the source's own call was unqualified, and qualifying it
would invent a name the source does not have.

| member | bytecode | pre-fix text | post-fix text |
| --- | --- | --- | --- |
| `boxed(I)Ljava/lang/Integer;` | `iload_0; invokestatic java/lang/Integer.valueOf:(I)Ljava/lang/Integer;; areturn` | `return valueOf(arg0);` — the pool's owner was dropped, and the text names nothing `Calls` declares | `return java.lang.Integer.valueOf(arg0);` |
| `local(I)I` | `iload_0; invokestatic Calls.own:(I)I; ireturn` | `return own(arg0);` | unchanged: `return own(arg0);` |
| `own(I)I` | `iload_0; ireturn` | `return arg0;` | unchanged: the member the two calls above reach |
| `<init>()V` | `aload_0; invokespecial java/lang/Object.<init>:()V; return` | `super();` | unchanged: the constructor javac emits, so the class is one a class file can be read from |

The two calls are the rule and its control:

* **`boxed`** is the shape the change is about. The pool entry's owner is `java/lang/Integer`, this
  class is not it, so the receiver is the owner written as Java spells a type: `/` becomes `.` and
  `$` is kept. The member's name stays the simple name — `valueOf` is not appended a descriptor, and
  `Integer.valueOf` is not folded into a cast, a `new Integer(…)`, or a `intValue` the bytecode does
  not contain.
* **`local`** is the control that keeps the rule honest in the other direction: the pool entry's
  owner here **is** `Calls`, which the presented body's own declaration states, so no qualifier is
  written. A text that named this class (`Calls.own(arg0)`) would be a call the source never wrote.

The owner is compared against the class the run's own member declaration states (`@declaration`'s
`this_class`, in internal form). A run that states no declaring class cannot prove the two differ
and writes the bare name it always wrote — it does not qualify every static call. An owner this
layer cannot spell as a Java type has no receiver to be, exactly as at the static field read and
write sites: the call is quoted with the diagnostic that names the owner rather than written without
one.
