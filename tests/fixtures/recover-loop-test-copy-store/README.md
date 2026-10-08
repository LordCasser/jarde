# The loop-test copy-and-store fixture (`recover-loop-test-copy-store`)

The same-form loop the io slice's registered boundary was made of — javac's `dup; store; test`
dance at a **loop's own test** — outside any guard body, beside the controls this slice's
admission is read against. Each class is compiled twice: `v8/` with javac 23.0.1
`--release 8 -g:none`, and `v8-javac8/` with the real javac 8 (Corretto 1.8.0_432) `-g:none`.
Neither leg has a `LocalVariableTable` or a `LineNumberTable`, so every name the recovery layer
writes is a derived `localN` name and every shape decision is a decision about control flow, not
about debug metadata.

`tests/recover_loop_test_copy_store.rs` reads the committed bytes; this README is the fixture's own
contract (the sources, the shapes, the commands, the digests and the recorded behavior).

## The anchor

`Probe.java` is the anchor, and `readAll` is the shape the slice exists for:

```java
    static String readAll(String path) throws IOException {
        FileReader fr = new FileReader(path);
        StringBuilder sb = new StringBuilder();
        int c;
        while ((c = fr.read()) != -1) { sb.append((char) c); }
        fr.close();
        return sb.toString();
    }
```

Its loop test is `aload_1; invokevirtual FileReader.read:()I; dup; istore_3; iconst_m1; if_icmpeq`
— the copy hands one value to the store and the other to the test, and the store's target is read
by the body — and it presents as `while ((local3 = local1.read()) != -1) { … }`, the store at the
test's own operand position. It is the io anchor's shape (`IO.readAll`) with the guard removed, so
the copy family's position is what the render measures; the guard itself is measured by
`tests/recover_io_resource_finally.rs` on the patrol's own `io.jar`.

`Probe.guardPlain` is the control the guard-body negative is read against: the same guard, the same
loop and an `if` inside the protected body, with **no** dance in the `if`. It presents — so the
refusal `guardIfFirst` keeps below is the dance's and not the guard body's own.

| member | shape | state |
| --- | --- | --- |
| `readAll` | `while ((c = fr.read()) != -1) { sb.append((char) c); }` — no guard | **presents**, on both legs and from a jar |
| `guardPlain` | `try { total = fr.read(); if (total > 0) { … } while ((c = fr.read()) != -1) { … } } finally { fr.close(); }` | **presents** (the guard body's own control) |
| `main` | the driver: `readAll`'s text with `\n` shown as `|`, then `guardPlain`'s sum | presents; the ignored replay runs it |

## The negatives

`ProbeControls.java` and the byte-patched `MultiCopy.class`; every class is verifier-valid
(compiled from source, or patched by one byte from one that was), so a refusal is evidence about
the admission rather than about damaged bytes.

| member | the link it breaks | the refusal it keeps |
| --- | --- | --- |
| `ProbeControls.parameterTarget` | the loop test assigns to a **parameter** (`int c`), whose declaration is the signature: the in-place expression is the copy family's *local* form | `// local 2 crosses a quoted fallback region …` at `// @bytecode 0 11 21 28` (the region layer's own refusal, at the dance's BCI) |
| `ProbeControls.guardIfFirst` | an `if`-**position** dance with an observable target inside the protected range: the assignment would be written in place there too, but the position is not the loop's own test | `// the local assignment condition was not completely proved` (the method quoted whole) |
| `MultiCopy` | the copy's **second consumer**: `Probe` with the loop test's `iconst_m1` (0x02) patched into a second `dup` (0x59) — same width, so no offset moves; the test then reads a copy of a copy | `// local 1 crosses a quoted fallback region …` at `// @bytecode 0 17 27 37` |

`MultiCopy.class` declares `Probe` (it is `Probe` with one byte moved), so the tests render it
under the name it declares. The pair is the point: one byte apart, `Probe.readAll` presents and
`MultiCopy.readAll` refuses.

`patch_controls.py` makes the patch, and it finds the site by the loop test's first four
instructions followed by the body's first three — `readAll`'s alone (`guardPlain`'s loop adds
`iload_2; iload_3; iadd; istore_2` instead) — reading the branch's own offset bytes rather than
matching them, so both legs patch alike. The patched class was loaded and run under
`java -Xverify:all` on both legs (`0/1105`, exit 0): the comparison compares two ints, and the
second copy is what the test consumes.

## The recorded digests and behavior

SHA-256 of the committed classes (`shasum -a 256`):

| file | digest |
| --- | --- |
| `v8/Probe.class` | `04793d304516e1fff4db46f9fe6075df0a6a0e073b7ba2eeee53d61164e72476` |
| `v8/ProbeControls.class` | `66368888136a841635f2a49ff70754f13fd46d335360bf6b7ecc5182415caec2` |
| `v8/MultiCopy.class` | `7a6235bceb67b728caa58d795b040ae7fbd15330bda99202de4877a12ca26f07` |
| `v8-javac8/Probe.class` | `a5e5ae43bb83b793f939a998b8cdc884fa6ea3f6f5ae0ac4dd141eed84cdd437` |
| `v8-javac8/ProbeControls.class` | `fb01167c4d9c66f8b922775c4ba47eb23a9a1b394ac25de1d308034dc4d91824` |
| `v8-javac8/MultiCopy.class` | `79f86b23fffe99d48046797a61bd907a4d69f4af0cb4cfbf7ebdd239b8c495b3` |

Measured behavior (`data.txt` is `hello\nworld\n`; `java -Xverify:all`, both legs):

* `Probe` (both legs) prints `hello|world|/1105` — `readAll`'s file read to EOF with the newlines
  shown as `|`, then `guardPlain`'s sum (the twelve bytes of the file plus one for the `if`);
* `ProbeControls` prints `1103/1105` — `parameterTarget`'s sum of the twelve bytes with its
  parameter's `-1` added, then `guardIfFirst`'s: the first byte through the `if`, the rest through
  the loop;
* `MultiCopy` (as `Probe`) prints `/1105` — its patched loop exits after the first read, which is
  the byte's own effect and not part of any claim.

## The commands

```
# v8 leg (javac 23.0.1)
javac --release 8 -g:none -d v8 Probe.java ProbeControls.java
# v8-javac8 leg (real javac 8)
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -g:none -d v8-javac8 Probe.java ProbeControls.java
# the multi-consumer control, per leg
python3 patch_controls.py v8/Probe.class v8/MultiCopy.class
python3 patch_controls.py v8-javac8/Probe.class v8-javac8/MultiCopy.class
```

`data.txt` is the driver's input on both legs, written where a run reads it.
