# 04 · Fixtures (task 1.2)

`tests/fixtures/recover-covariant-array-store-receiver/` — four sources, two compiler legs, sixteen
files. The full README with the sources' digests, the class files' digests, the two legs' commands
and the discriminating `javap` facts is
[tests/fixtures/recover-covariant-array-store-receiver/README.md](../../../../tests/fixtures/recover-covariant-array-store-receiver/README.md).

| fixture | role | pinned by |
| --- | --- | --- |
| `AS` | the patrol's own fixture, **verbatim** (`db31a5d8…`, byte-identical to `fixture/AS.java`) | the anchor: `storeWrong`/`storeNumber` widen, `storeRight` does not; ignored replay `s`/`ASE1`/`ASE2` |
| `SD` | the behavior driver: read-back, `Object`-component and `null` controls, three caught covariant stores (type + throwing method), the primitive-array negative, the same-type element receiver | the behavior comparison the acceptance demands |
| `UB` | the unproven receiver (two-branch local) and the stated-component `checkcast` receiver | the negative: presentation kept, stripped text stays uncompilable |
| `SC` | the subtype value (`String` into `CharSequence[]`) | the admission's boundary, pinned rather than implicit |

The two legs:

```sh
javac --release 8 -Xlint:-options -d v8 AS.java SD.java UB.java SC.java
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -d v8-javac8 AS.java SD.java UB.java SC.java
```

`v8/AS.class` is byte-identical to the patrol's `fixture/as.jar` entry
(`2369235f…f286f6`), so the anchor's bytes are the patrol's own. The two legs present identical
texts for every member except `UB.throughObject` (the real javac 8 materializes the `checkcast`
twice, javac 23 folds it once), which the test pins per leg.

The replay protocol (ignored tests) is the patrol's own strip — comment lines dropped — with both
compilers and both JVMs: the stripped text is compiled by the leg's own `javac` and run under
`-Xverify:all` beside the fixture's own class files, and the two runs' outputs (and exception
identities) are compared. The real javac 8 path is asserted to exist before it is used.
