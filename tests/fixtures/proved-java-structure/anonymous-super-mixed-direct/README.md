# Mixed superclass arguments and capture, direct-return anchor

This Java 8 fixture is the mixed-parameter anonymous-class anchor of
`recover-anonymous-mixed-super-capture`: the anonymous child of `Base` receives two real
superclass constructor arguments (`text(…)`, `number(…)`) and captures one local (`captured`)
into a synthetic `val$captured` field. The original bytecode stores the capture **before**
`invokespecial Base.<init>`, so the child's physical constructor is not compilable Java 8 source
when presented verbatim; the source-level `new Base(args…) { … }` expression lets javac rebuild
that pre-super store itself.

The frozen `.class` files were produced with:

```sh
javac --release 8 -g:none -d . AnonymousSuperMixedDirect.java
```

`Base` carries a second, unused `(int, String)` constructor so the reordered-super-arguments
refusal negative (see `../anonymous-super-mixed-refusals/`) can run under `java -Xverify:all`.

Original behavior under `java -Xverify:all` (the event log is the observable contract; the last
line proves the body read the captured value, not `null`):

```text
capture|arg:super-label|arg:super-value|base:explicit:17
explicit:17
capture|arg:super-label|arg:super-value|base:explicit:17|captured
```

After the mixed projection the complete source set is exactly `AnonymousSuperMixedDirect.java`
plus `Base.java` (the `$1` child stays queryable but is not part of the source set, exactly like
the delivered interface-form fixtures). `javac --release 8` compiles that set and
`java -Xverify:all` reproduces the event log line for line. Evidence:
`openspec/evidence/java-syntax-2026-10-04/recover-anonymous-mixed-super-capture/`.
