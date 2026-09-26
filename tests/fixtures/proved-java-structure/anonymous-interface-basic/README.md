# Anonymous interface implementation, one site

This fixture isolates the smallest DT-05 shape: a top-level interface and one
uncaptured anonymous implementation at one allocation site. It has no instance
initializer, nested anonymous class, or explicit constructor arguments.

The checked-in `.class` files are frozen outputs of:

```sh
javac --release 8 -g:none -d . I.java AnonymousInterfaceBasic.java
```

Run the frozen input with:

```sh
java -Xverify:all -cp . AnonymousInterfaceBasic
```

Expected output:

```text
7
```

`class-sha256.txt` and `javap.txt` are the frozen identity and bytecode record.
The full JADX/Jarde source comparison and Java 8 recompilation are recorded in
`openspec/evidence/java-syntax-2026-09-27/anonymous-interface-basic/`.
