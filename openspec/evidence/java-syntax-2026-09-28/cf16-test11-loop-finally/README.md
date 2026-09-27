# CF16 TestTryCatchFinally11 finally-loop baseline

This fixture freezes the pinned JADX `TestTryCatchFinally11.TestCls.test(List)` case and a small behavioral probe with the same loop/finally bytecode shape. It records Jarde's method-level explanation-only marker, whose displayed message says the graph is not reducible over blocks `[48, 58]`. A compilable presentation is not evidence that the recovered method preserves behavior. Whether a plain-transfer irreducibility check rejects this handler loop before the FINALLY guard is a hypothesis to verify; this evidence does not attribute the refusal to a particular CFG stage.

`pinned/TestTryCatchFinally11.java` is the full JADX integration test source at commit `2fb1b16386941660fda07e9017285aec40fcb37f`, SHA-256 `3e077dc69f9325fa55dec9ba92ba14a508b2ae4e6ce3d05aeb3462f9c2df1bf5`. `TestTryCatchFinally11$TestCls.class` is the exact nested class from that pinned checkout's test build, SHA-256 `3a67a7f63596c5ca7b7dbdb97a2027224d69f2a1d7b21e4f598d897510bc5f92` (class-file major 55). The replay rejects a changed source, class, or JADX checkout. `javap-TestCls.txt` records the pinned bytecode: `test(List)` spans BCI 0–79, with exception rows `[0,4) -> 38 any` and `[38,40) -> 38 any`.

The pinned class is loaded from the original class file and from complete JADX and Jarde source presentations; both recovered sources compile with `javac --release 8`. A small assertion-helper stand-in only resolves the unrelated reference in `check()`, while the runner calls `test(List)` directly. The separate `FinallyLoop` probe provides two observable paths: normal return and an exception from the protected body. Its Java 8 bytecode has the same BCI layout and exception rows. The only opcode differences from the pinned test method are the private helper call opcodes at BCI 1, 29 and 70 (`invokespecial` in the Java 8 probe, `invokevirtual` in the pinned class); the replay checks and records those explicitly.

Original and JADX sources preserve both probe outputs: `ok:102` and `throw:102:body`. The Jarde presentation compiles and passes JVM verification, but its `test(List)` body is empty under the explanation-only marker. Both invocations therefore print `ok:0`, including the invocation that requests a body failure: no body code runs to throw it. This is the observed consequence of compiling a presentation whose method is explicitly marked unrecovered; it is not presented as a behavior-correct decompilation. The pinned target class is also replayed through normal and empty-list paths (`two:102`, `empty:100`); JADX preserves both, while Jarde's marked empty body yields `two:0`, `empty:0`.

Replay with a Jarde CLI path; `JADX` may override the default pinned launcher, and an optional second argument retains generated outputs:

```sh
openspec/evidence/java-syntax-2026-09-28/cf16-test11-loop-finally/replay.sh /path/to/jarde-cli /tmp/cf16-test11-replay
```

The script does not build Jarde or JADX. It uses `javac --release 8` and runs compiled sources with `java -Xverify:all`. The recorded baseline CLI was built from Jarde `f1228c56a8be573328662821c96a14af8cd98c97`; replay records the supplied CLI path, version and binary SHA rather than requiring that same build.
