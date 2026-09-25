# Generic enclosing member call-site classes

These Java 8 class files are the `-g:none` variant of the source fixture at
`openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/shape-matrix/fixture`.
They are test inputs for the selected `Outer.A<T>` relation, member target path, raw and parameterized
method headers, and same-run member-creation return candidate. The independent `accept-jarde.py`
check rebuilds both `-g` and `-g:none` callers against a separately compiled original `Outer` jar.

To reproduce these inputs, compile `Outer.java` and the four `Use*.java` files with
`javac --release 8 -g:none -d <output>`. `Runner.java` is omitted because these unit tests do not run
the fixture.

The committed binary checksums are:

| Class | SHA-256 |
| --- | --- |
| `Outer.class` | `caa5b1470a7e8013ce849b6ef132672829a9d7da125336815b4fa4eba2af5b43` |
| `Outer$A.class` | `954faae0aff34c6627fbb63a4beb9bc99af7b316bb0112f685828fbdebde3459` |
| `Outer$A$Plain.class` | `9e74f651389f27d9aee43b3abfa7ed4eacf38b8b8cdda5b8aec6a2829574311c` |
| `Outer$A$Generic.class` | `01caef44567e8bb2979bfc734711386ec00a8eca8b65e92b8da872ba0585e6f7` |
| `UsePlainRaw.class` | `6e8986a85b46bc667c211ff490c9ea885ef66eecfeb93c223512cfc17b465408` |
| `UsePlain.class` | `45f2bc8aeab0561a20212dd31dd62ca7f781cbadb1bed79a033e20a31f173727` |
| `UseGenericObject.class` | `6af4891b032f2751595c86b34ede87ad63bf22532944a6040f71296667af309c` |
| `UseGenericTyped.class` | `8cc6983f428336633637f17e71c74c54cf24e5c1816921a2bc95fb3ae24327c4` |
