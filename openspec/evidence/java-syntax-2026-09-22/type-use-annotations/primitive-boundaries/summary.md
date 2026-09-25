# Observed result

Environment: OpenJDK `23.0.1` (`javac 23.0.1`, `javap 23.0.1`) using
`javac --release 8`; the resulting classfile major version is 52 (Java 8).
`ScalarCases.java` uses one runtime annotation declared with
`@Target({FIELD, METHOD, PARAMETER, TYPE_USE})`, applied as ordinary `@A int` to
field `field`, return type of `answer`, and parameter `value` of `echo`.

## Physical classfile evidence

`javap -v -p` in `build/baseline.javap.txt` reports the ordinary source spelling as
two attributes at each site:

| Site | Declaration attribute | Type annotation target |
| --- | --- | --- |
| `int field` | field `RuntimeVisibleAnnotations` | `RuntimeVisibleTypeAnnotations`, `FIELD` |
| `int answer()` return | method `RuntimeVisibleAnnotations` | `RuntimeVisibleTypeAnnotations`, `METHOD_RETURN` |
| `int echo(int)` parameter 0 | method `RuntimeVisibleParameterAnnotations` | `RuntimeVisibleTypeAnnotations`, `METHOD_FORMAL_PARAMETER`, `param_index=0` |

The patch report (`build/patched/ScalarCases.patch-report.txt`) confirms that it
deleted exactly those three member attributes. `build/patched.javap.txt` shows all
three `RuntimeVisibleTypeAnnotations` entries remain and the corresponding
declaration attributes are absent. The class-level attributes and the annotation
type `A` remain unchanged.

Class SHA-256 values:

| File | SHA-256 |
| --- | --- |
| Baseline `ScalarCases.class` | `57c80b5db210e50fde0c5d29f5e6ab85875ad0b4ea2a48fc57fa770e169806ec` |
| Patched `ScalarCases.class` | `0120bf25b0d225379ddbf3844881160cfc822d65a34db5f1df15ac6dffd72127` |

## Verification and code stability

Both the baseline and patched classes load while the reflection program is run
with `java -Xverify:all`. For the patched class, reflection reports:

```text
field.declaration=[]
field.type=[A]
answer.declaration=[]
answer.returnType=[A]
echo.parameter=[]
echo.parameterType=[A]
```

Thus the JVM accepts all three type-only primitive annotations. The patch report
compares SHA-256 hashes of the complete raw `Code` attribute payloads, including
code and nested code attributes. Constructor, `answer`, and `echo` are each
`IDENTICAL` before and after; their digests are recorded in
`build/patched/ScalarCases.patch-report.txt`.

## Source-expression boundary

Recompiling the fixture with the same annotation declaration and ordinary
`@A int` spellings regenerates the baseline attributes: each site has both the
applicable declaration annotation attribute and its type annotation attribute.
For this fixed annotation type, ordinary Java source cannot recreate the patched
state in which the annotation is present only on the primitive type-use site,
because the same source occurrence is also applicable to the field, method, or
parameter declaration. This is a scoped result about that overlapping `@Target`
and spelling; it does not claim that primitive type-use annotations in general
cannot be written in Java source (a distinct `TYPE_USE`-only annotation can be).
