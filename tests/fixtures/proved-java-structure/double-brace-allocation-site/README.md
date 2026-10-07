# Double-brace allocation point: one control, three negatives, both legs

The frozen shapes of change `recover-double-brace-allocation-site`. The anchor itself is the
patrol's own `DB` (`openspec/evidence/java-syntax-2026-10-05/double-brace-patrol/fixture/db.jar`,
and the `double-brace-capture/` fixture's two legs); this directory freezes the shapes the
four-criteria admission must **refuse**, each differing from the positive control in exactly one
criterion, so the gating experiment can be read off one fixture set:

| case | classes | which criterion fails | presentation |
| --- | --- | --- | --- |
| `controls/single-site/` | `DBS`, `DBS$1` | none — the control | the double-brace form at the allocation point |
| `negatives/methods/` | `DBM`, `DBM$1` | the companion declares `void mark()` | the companion's own class and `new DBM$1(s)` |
| `negatives/unspellable-super/` | `DBN`, `DBN$1`, `Carrier`, `Carrier$Nested` | the superclass `Carrier$Nested`'s pool form is not a source name | `new DBN$1(s)` |
| `negatives/multi-site/` | `DBS2`, `DBS2$1` | the companion is constructed **twice** | `new DBS2$1(arg0)` at both sites |

The control's source is the double-brace shape itself:

```java
public class DBS {
    static List<String> make(String s) { return new ArrayList<String>() {{ add(s); }}; }
    public static void main(String[] a) { System.out.println(make("z").get(0)); }
}
```

## The hand-made host (`negatives/multi-site/`)

No Java source states the multi-site shape: one anonymous class body is one allocation, so a
companion constructed twice exists only as bytes. `freeze.py` compiles the child from the control's
source (renamed `DBS2`) and writes the host itself — `make(Ljava/lang/String;)Ljava/util/List;`,
which the child's own `EnclosingMethod` names, and `again(Ljava/lang/String;)Ljava/lang/Object;`,
one `new DBS2$1(arg0)` site each:

```text
new; dup; aload_0; invokespecial DBS2$1.<init>(Ljava/lang/String;)V; astore_1; aload_1; areturn
```

Two **methods** rather than two statements in one, so the allocation point's own one-site
discipline is not what refuses the shape: each method holds exactly one verified allocation, and
the criterion under test is the owner census's single use. The host's `InnerClasses` row is copied
out of the compiled host's own attribute (class, no outer class, no inner name, the same flags), so
the child's own row and this one agree. The frozen host loads under `java -Xverify:all` (it has no
`main`, which is the run's own message).

## Legs

Both legs of every case are frozen, because the class flag javac 8 sets on an anonymous class
(`final class DBS$1` against `class DBS$1`) is a fact the admission reads:

| leg | compiler | command |
| --- | --- | --- |
| `v23/` | the ambient javac, `--release 8` | `javac --release 8 -g -Xlint:-options -d v23 …` |
| `v8/` | a real javac 8 | `<jdk8>/bin/javac -g -Xlint:-options -d v8 …` |

`JARDE_JAVAC8` names the JDK 8 install (a directory or the `javac` binary) and defaults to the
Corretto install this repository's evidence records.

```text
python3 tests/fixtures/proved-java-structure/double-brace-allocation-site/freeze.py
```

The script recompiles every case with both compilers, generates the hand-made host per leg, and
writes `SHA256SUMS`. The guards are `tests/recover_double_brace_allocation_site.rs` (the control's
double-brace form, the three negatives' byte-identical path-A text, and the control's recompiled
run) and `tests/double_brace_capture.rs` (the patrol anchor).
