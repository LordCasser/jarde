# P3 fixture: the member forms a class file's own flags decide

`v8/Shape.class` and `v8/Holder.class` are real compiled samples: the sibling `Shape.java` and
`Holder.java` compiled by **javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Shape.java Holder.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes both classes anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the samples are committed as bytes.

| property | `Shape` | `Holder` |
| --- | --- | --- |
| class-file version | 52.0 (Java 8) | 52.0 (Java 8) |
| class flags | `0x0601` (`public`, `interface`, `abstract`) | `0x0031` (`public`, `final`, `super`) |
| bytes | 191 | 359 |
| SHA-256 | `91f9a3a0f60fcf6a510c826b87b8e3b484e168d7281d3a1a8f8cc4b01b94f32e` | `11cbb8c01e738c5d2c4959b09fbba29e3949e4aff9445970154c87ce1d25b5ea` |
| debug attributes | none (`-g:none`): every slot is named by its ordinal | none (`-g:none`) |
| read by | `tests/p3_declaration_handoff.rs` (the declaration forms through `Engine::recover_method`) | the same file |

The two classes are one sample because the question they answer needs both: a `default` method is not
a flag. JVMS 4.6 gives an interface's methods `public`, `static`, `abstract` and the rest, and "the
`default` keyword" is exactly *an interface's method that is neither `static` nor `abstract`* — so
`Shape.scaled` and `Holder.value` carry the same `public` flag and are two different declarations.
Only `ACC_INTERFACE` on the class that declares them tells them apart, which is why the class's own
access flags have to travel with the member's.

## What each member is for, and the bytes that make it

### `Shape` (the interface)

```text
public abstract int sides();        // flags 0x0401 (public, abstract); no `Code` attribute at all
public default int scaled(int);     // flags 0x0001 (public) — the class's `ACC_INTERFACE` is what
     0: aload_0                     // 2a b9 00 01 01 00 1b 68 ac   makes this a `default` method
     1: invokeinterface #1,  1       // InterfaceMethod sides:()I
     6: iload_1
     7: imul
     8: ireturn

public static int sum(int, int);    // flags 0x0009 (public, static)
     0: iload_0                     // 1a 1b 60 ac
     1: iload_1
     2: iadd
     3: ireturn
```

`sides()` is the member with **no body**: an interface's `abstract` method declares no `Code`, so a
request for it is a member the class file declares without a body, and no declaration is published
for it (a run that read no member header states no class facts either). `scaled(I)I` is the
`default` method and `sum(II)I` the interface's `static` one — the two forms that need the class's
`ACC_INTERFACE` and cannot be told apart from `Holder`'s members by flags alone.

### `Holder` (the class)

```text
static final Object TOKEN;
private final int value;

public Holder(int);                 // flags 0x0001 (public)
     0: aload_0                     // 2a b7 00 01 2a 1b b5 00 07 b1
     1: invokespecial #1            // Method java/lang/Object."<init>":()V
     4: aload_0
     5: iload_1
     6: putfield      #7            // Field value:I
     9: return

public int value();                 // flags 0x0001 (public)
     0: aload_0                     // 2a b4 00 07 ac
     1: getfield      #7            // Field value:I
     4: ireturn

public static Holder of(int);       // flags 0x0009 (public, static)
     0: new           #8            // class Holder
     3: dup                         // bb 00 08 59 1a b7 00 0d b0
     4: iload_0
     5: invokespecial #13           // Method "<init>":(I)V
     8: areturn

static {};                          // flags 0x0008 (static)
     0: new           #2            // class java/lang/Object
     3: dup                         // bb 00 02 59 b7 00 01 b3 00 10 b1
     4: invokespecial #1            // Method java/lang/Object."<init>":()V
     7: putstatic     #16           // Field TOKEN:Ljava/lang/Object;
    10: return
```

`Holder`'s four members are the other four forms: a **constructor** (the `<init>` of JVMS 2.9), an
ordinary **instance method**, an ordinary **static method**, and a **static initializer** — a real
`<clinit>`, because `TOKEN` is initialised by a call rather than by a constant (`static final Object
TOKEN = new Object();` would not be inlined by `ConstantValue`, and does not need to be). The
constructor is also the one shape whose *body* needs the class: `putfield #7` at BCI 6 stores through
the uninitialized `this`, and JVMS 4.10.1.9 lets that write reach only a `Fieldref` that names the
class being constructed — so `field@1` can read the write at all only when the class's own name is
known.

## Reproducing

```text
cd tests/fixtures/p3-declaration
mkdir -p v8
javac --release 8 -g:none -d v8 Shape.java Holder.java
shasum -a 256 v8/Shape.class  # 91f9a3a0f60fcf6a510c826b87b8e3b484e168d7281d3a1a8f8cc4b01b94f32e
shasum -a 256 v8/Holder.class # 11cbb8c01e738c5d2c4959b09fbba29e3949e4aff9445970154c87ce1d25b5ea
```

The listing above is `javap -p -c -constants v8/Shape.class` and `javap -p -c -constants
v8/Holder.class`, with the BCIs kept; the hex column beside each member is that member's own `Code`
array as the class file stores it, read out of the attribute whose length `javap` reports.
