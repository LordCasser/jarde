# Java 8 bridge projection boundaries

These three cases pin the class-level conditions for projecting a compiler bridge from complete
Java source. Each runner is source-only; compiled runner classes are temporary outputs.

| Case | Frozen class input | SHA-256 | Code attributes | `java -Xverify:all` output |
| --- | --- | --- | ---: | --- |
| `positive/` | `v8/BridgeProbe.class` (with `v8/BridgeApi.class`) | `03499dad67f54ce855af6f102b61c19916f72718a6d8756a35b019e3cda2af2f`; API `6452b573128f0943f407b4408704d8238fe7a9f1b50450c15d900290f9f0012a` | 4 | `value|value|value` |
| `negative/` | patched `v8/FakeBridge.class` | `1a51179c7d05a89d71b46e86dd010bc9a4f74aeae6834a56a7e76fc17bf1a420` | 4 | `value|value|1` |
| `orphan/` | patched `v8/OrphanBridge.class` | `68262cf87e3242aa0a8d4927caab3f30f9ce9911e2ae1d9290f06beb66a2de0f` | 3 | `value|value` |

The compiler is javac 23.0.1 with `--release 8 -g -Xlint:-options`. `BridgeProbe` implements
`BridgeApi<String>` and javac emits `ACC_BRIDGE|ACC_SYNTHETIC Object get()` forwarding to
`String get()`. `FakeBridge` and `OrphanBridge` compile ordinary `geh():Object` methods; the
documented patch changes the equal-length constant-pool name to `get` and adds bridge/synthetic
flags. The FakeBridge body retains its `calls++` side effect. The orphan body is a pure forward
but has no inherited erased method requiring regeneration.

To reproduce all frozen bytes in a temporary directory from this directory:

```sh
tmp=$(mktemp -d)
mkdir -p "$tmp/positive" "$tmp/negative" "$tmp/orphan"
javac --release 8 -g -Xlint:-options -d "$tmp/positive" positive/BridgeProbe.java positive/BridgeRunner.java
javac --release 8 -g -Xlint:-options -d "$tmp/negative" negative/FakeBridge.java negative/FakeRunner.java
python3 negative/patch.py --input "$tmp/negative/FakeBridge.class" --output "$tmp/negative/FakeBridge.class.patched"
cp "$tmp/negative/FakeBridge.class.patched" "$tmp/negative/FakeBridge.class"
javac --release 8 -g -Xlint:-options -d "$tmp/orphan" orphan/OrphanBridge.java orphan/OrphanRunner.java
python3 negative/patch.py --input "$tmp/orphan/OrphanBridge.class" --output "$tmp/orphan/OrphanBridge.class.patched"
cp "$tmp/orphan/OrphanBridge.class.patched" "$tmp/orphan/OrphanBridge.class"
sha256sum "$tmp/positive/BridgeProbe.class" "$tmp/positive/BridgeApi.class" "$tmp/negative/FakeBridge.class" "$tmp/orphan/OrphanBridge.class"
java -Xverify:all -cp "$tmp/positive" BridgeRunner
java -Xverify:all -cp "$tmp/negative" FakeRunner
java -Xverify:all -cp "$tmp/orphan" OrphanRunner
```

The negative patcher is deliberately shared: it finds the one `geh` method, renames it to `get`,
and sets `ACC_BRIDGE|ACC_SYNTHETIC` only for the matching `()Ljava/lang/Object;` method. The
commands above recompile the originals, patch both boundary classes, and run all source-only
runners. Compare SHA-256 and output against the table. The positive bridge's erased entry can
additionally be checked with
`javap -p -c -v` and a raw `BridgeApi` call. The negative runner uses `MethodHandles` to invoke
`get()Object`, making the side effect observable; the orphan runner uses the same erased lookup
to prove that deleting a pure-forward bridge loses a real JVM entry point.
