# P3 fixture: an annotation type's members declare their defaults in the class file

`v8/Marker.class` and `v8/Kind.class` are one real compiled sample: the sibling `Marker.java`
compiled by **javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Marker.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the classes anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

`Marker.java` declares a top-level `@interface` and a top-level `enum` beside it, so one command
writes **two** class files. That is deliberate for this one task: the `e` element value's type index
is a descriptor (`LKind;`) that has to name a class the same pool holds, so the enum constant needs a
real class file to be a real class. Both are committed here.

| property | `Marker` | `Kind` |
| --- | --- | --- |
| class-file version | 52.0 (Java 8) | 52.0 (Java 8) |
| bytes | 448 | 725 |
| SHA-256 | `b8ed6e5d205331693e835e88afe13b3d739ac780075f8b4d7ad936640aaff9be` | `320bbda363c188a554e37940cd1a7841f0722e7296dafcd8b7b36907cca11e25` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`) | none (`-g:none`) |
| read by | `tests/p3_annotation_default.rs` (all eight member declarations, and the one member that must stay without a default) | the same test (presented as its own class, with its two constants) |

## What the defect is, and what each member is for

An annotation type's default is not a body and not an inference: `javac` writes one
`AnnotationDefault` attribute (JVMS 4.7.22) on every member that declares a default, and the class
file's own bytes there are the declaration (`javap -v` prints them below as `default_value`). The
presentation wrote `public abstract java.lang.String value();` — the default was silently absent,
which is the one thing this presentation must never do.

| member | `default_value` | pre-fix text | post-fix text |
| --- | --- | --- | --- |
| `value()` | `s#10` (`"x"`) | `public abstract java.lang.String value();` | `public abstract java.lang.String value() default "x";` |
| `count()` | `I#13` | `public abstract int count();` | `public abstract int count() default 1;` |
| `flag()` | `Z#13` | `public abstract boolean flag();` | `public abstract boolean flag() default true;` |
| `big()` | `J#18` (`5l`) | `public abstract long big();` | `public abstract long big() default 5L;` |
| `grade()` | `C#22` (`65`) | `public abstract char grade();` | `public abstract char grade() default 'A';` |
| `kind()` | `e#25.#26` (`LKind;.ONE`) | `public abstract Kind kind();` | `public abstract Kind kind() default Kind.ONE;` |
| `pair()` | `[I#29,I#30` | `public abstract int[] pair();` | `public abstract int[] pair() default {3, 4};` |
| `ratio()` | `D#33` (`0.5d`) | `public abstract double ratio();` | unchanged: a `double` constant's digits are a spelling rule of their own (2c.5), so no default is invented for it |

Each member is one tag of the attribute, and each is a control for the others:

* **`value`** is the `s` tag: a `CONSTANT_Utf8` holds the string, and the text writes it as a Java
  string literal through the one escaping rule this engine escapes string literals with.
* **`count`, `flag`, `grade`** are the `I`, `Z` and `C` tags over one `CONSTANT_Integer`: the same
  pool entry is written `1`, `true` and `'A'`, so the text is decided by the tag the attribute
  states and never by the number alone.
* **`big`** is the `J` tag: the digits plus the `L` its type needs.
* **`kind`** is the `e` tag: the type index is the descriptor `LKind;` the pool holds and the
  constant's own simple name is `ONE`, so the text is `Kind.ONE` — never a quoted string, which is
  what the same two pool entries would be if the tag were `s`.
* **`pair`** is the `[` tag: an array of two elements, written `{3, 4}`.
* **`ratio`** is the `D` tag, and it is the control in the other direction: the attribute is
  declared and its value is not spelled this round, so the member keeps the declaration the flags
  and the descriptor state. A text that wrote `default 0.5` would be inventing digits, and a text
  that dropped the member's other facts would be hiding the attribute — neither is this
  presentation's.

No member here declares a body: an annotation type's members are `abstract` and carry no `Code`
attribute, so the change touches declarations only. `Kind` is presented as its own class, exactly as
the class file states it: an `enum` with the two constants its `<clinit>` builds.
