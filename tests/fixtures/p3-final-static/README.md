# P3 fixture: blank `static final` writes in one Java 8 initializer

`v8/FinalStaticProbe.class` is the only permanent class file.  It was compiled from
`FinalStaticProbe.java`, `FinalStaticSupport.java`, and `FinalStaticRunner.java` with:

```text
javac --release 8 -g:none -d /tmp/jarde-final-static-20260923 \
  FinalStaticSupport.java FinalStaticProbe.java FinalStaticRunner.java
```

The helper and runner are source-only inputs for the ignored JVM comparison.  The runner receives
`true` or `false`; each invocation is a new JVM, so both arms of `<clinit>` are executed.  Its
original-side class is replaced with the committed `FinalStaticProbe.class` after compiling the
source-only files.

| property | value |
| --- | --- |
| class | `FinalStaticProbe` |
| class-file version | 52.0 (Java 8) |
| bytes | 1089 |
| SHA-256 | `2bdb603ff8b3638d48256163fb729a0d5ba34a07a65161221b339ecb7ce59a45` |
| fields | 8 (seven static, one instance final) |
| methods with `Code` | 5 (`<init>`, `instanceValue`, `readAfterAssign`, `snapshot`, `<clinit>`) |
| debug attributes | none (`-g:none`; the `seed` source local is recovered from its slot ordinal) |

The original class passes `java -Xverify:all` in both branches.  The `true` JVM prints:

```text
1:2:5:3:4:8:17
first,second,local,local0_2,branch-true
read=8
instance=41
```

The independent `false` JVM prints:

```text
1:2:5:3:4:8:17
first,second,local,local0_2,branch-false
read=8
instance=41
```

The two first calls and the two branch calls are all in the same existing `<clinit>` region.  The
class also keeps `constantValue = 17` as a `ConstantValue` field and `instanceValue` as a constructor
written final field, so their access forms remain controls for the blank static-final cases.
