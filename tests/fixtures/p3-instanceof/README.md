# P3 fixture: Java 8 `instanceof` expressions

`v8/InstanceOfProbe.class` is the only permanent class file.  The source-only helper and runner
are compiled in a temporary directory for the ignored whole-class comparison:

```text
javac --release 8 -g:none -d /tmp/jarde-instanceof-20260923 \
  InstanceOfSupport.java InstanceOfProbe.java InstanceOfRunner.java
```

The probe deliberately contains only positive, compilable methods.  It covers Object and `null`
inputs, ordinary class and interface targets, primitive/reference/multidimensional arrays,
`String` widened to `Object` before an `Integer` test, a side-effecting `String` producer widened to
`Object` before a test (including the producer's exception), a real `Runnable` method reference,
and boolean consumption by a local, a call parameter, and an ordinary `if`.  The unsupported 0/1
merge and discarded/duplicate-consumer cases are not part of this whole-class fixture.

| property | value |
| --- | --- |
| class | `InstanceOfProbe` |
| class-file version | 52.0 (Java 8) |
| bytes | 1548 |
| SHA-256 | `d8f4a437d0cd0f3ec12672791f27b1c11cfbcdc6885d10eddcd3114aa0838e56` |
| fields | 0 |
| methods with `Code` | 15 (constructor, 12 public probes, `keep`, `empty`) |
| debug attributes | none (`-g:none`) |

`javap -c -v` shows `instanceof` in every probe shape.  `widenedString` has no `checkcast` before
its `Integer` test; `called` has the same no-checkcast widening after `InstanceOfSupport.value()`;
`functional` has `invokedynamic ... ()Ljava/lang/Runnable;` immediately followed by its
`instanceof Runnable`.

With `java -Xverify:all`, the frozen original class prints:

```text
string-text=true
string-null=false
string-number=false
null=false
runnable=true
runnable-null=false
primitive-array=true
primitive-array-string=false
reference-array=true
reference-array-object=false
multi-array=true
multi-array-one=false
widened-string=false
local-true=true
local-false=false
parameter-true=true
parameter-false=false
branch-true=1
branch-false=0
functional=true
called=false:1
called-fail=java.lang.IllegalStateException:1
```

The ignored Rust test first compiles the source-only helper and runner, replaces the generated
`InstanceOfProbe.class` with the frozen bytes, and runs that original side.  It then compiles the
complete Engine class-source text with the same helper and runner and compares the output.  No
generated method is removed or replaced.
