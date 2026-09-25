# `ireturn` descriptor narrowing audit

This is a source-only audit of an `ireturn` whose method descriptor is narrower than the JVM int-shaped stack value. It writes only this evidence directory. No production code, Rust test, Cargo build, permanent fixture, census, fingerprint, or OpenSpec planning file was changed.

The four source classes are deliberately identical apart from their class names:

```java
public static int run(int value) { return value; }
```

Each source was compiled with `javac --release 8 -g:none`. The saved `run_audit.py` then replaced the one and only constant-pool UTF8 `(I)I` descriptor with `(I)B`, `(I)C`, `(I)S`, or `(I)Z`. The patch is byte exact: one descriptor byte changes, the class length stays fixed, and both `Code` attribute payload hashes remain unchanged. The patched class was compiled against a dedicated runner and executed with `java -Xverify:all`; the runner contains all 13 required boundary inputs:

`Integer.MIN_VALUE`, `Integer.MAX_VALUE`, `-65537`, `-32769`, `-129`, `-2`, `-1`, `0`, `1`, `2`, `128`, `32768`, `65535`.

The direct patched-class oracle produced 13 lines for each class, 52 lines total. The expected semantics were observed exactly:

* `B`: signed low eight bits, equivalent to `(byte) value`.
* `C`: unsigned low sixteen bits, equivalent to `(char) value` and printed through an `int` cast.
* `S`: signed low sixteen bits, equivalent to `(short) value`.
* `Z`: `value & 1`, printed as `false`/`true` by the Java caller.

## Patched class facts

Every class has two `Code` attributes (constructor plus `run`) and exactly one `ireturn` in `run`. The source and patched `Code` payload hashes are the same in every `patch.json`.

| Class | Descriptor | Source bytes / SHA-256 | Patched bytes / SHA-256 | descriptor byte | Code attributes | `ireturn` in `run` |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `ReturnByte` | `(I)B` | 166 / `f21cbdbb27d5a206d038b08848b40ef3a04637f5383a9f47e0b332f9bb81d9a3` | 166 / `9c6b934364abd533d14e11c8c6a051cf10e4de2642cddc2acaf52ada8be55505` | offset 92 | 2 | 1 |
| `ReturnChar` | `(I)C` | 166 / `89c6fc1cbc8899085a9122384cf2ec9be61829fa3a4dd621adb75332aed21458` | 166 / `a07ecdb8c34f43aa1aa0a64c7da8713632a05a24108cda2afd65386952ac592e` | offset 92 | 2 | 1 |
| `ReturnShort` | `(I)S` | 167 / `ba4fe8929d9e066df7451171ece34f2a8b9de977fc8a8cfb6f27ea4d75d536aa` | 167 / `54a15fee9bdc325a7df693acd131807557480ac63cc26f2b24c3269e06d1f3e5` | offset 93 | 2 | 1 |
| `ReturnBoolean` | `(I)Z` | 169 / `7e795070a3ed30801a56caa3d1797806f2f019cba3f002bd42777459fa0de37c` | 169 / `95b351c13d04522dca82259b1b20bc5c48223ac0084a6fad4e61044a849e2b9d` | offset 95 | 2 | 1 |

The exact changed range, descriptor occurrence count, source/patched hashes, and the equal before/after `Code` hashes are in each class's `patch.json`. `javap-before.txt` and `javap.txt` preserve the complete pre-patch and patched class views.

## Three-way output results

The CLI was measured before and after the audit. Both files contain SHA-256 `feed5c377a439490efb12a2a46bdfa0e94c32dffd3a83bd1583b70fe89317350`; the CLI did not change during the run. Java is OpenJDK `23.0.1`; JADX is `1.5.6`.

The direct patched class is the only execution oracle that compiled and ran for every case. JADX produced complete source for all four classes, but each source uses the int-shaped expression directly in the narrower return:

```java
public static byte run(int i) {
    return i;
}
```

`javac --release 8` rejects the byte/char/short variants as lossy `int` to narrow conversion and rejects the boolean variant as `int` to `boolean`. Therefore there is no JADX execution comparison to claim; the exact compiler diagnostics are in each `jadx-javac.log`.

The current jarde CLI returned a complete report for all four patched classes and produced one `@bytecode` quote for each `run` method. The generated signatures are `(I)B`, `(I)C`, `(I)S`, and `(I)Z`, but the `run` body has no recovered statement, so each unchanged full class fails `javac --release 8` with `缺少返回语句`. No jarde execution was attempted after that compile failure. The complete text, report, and compiler log are kept per class in `jarde.java.txt`, `jarde-report.txt`, and `jarde-javac.log`.

This is a semantic refusal rather than an invalid-bytecode artifact: all four patched classes passed `-Xverify:all`, the only changed class-file fact is the method descriptor, and the JVM itself returned the expected low bits through `ireturn`.

## Smallest implementation reading

The current decode maps `ireturn` to the existing `Operation::Return` path. `build::return_expr` already receives the member descriptor as `return_type` and renders the value at the actual return consumer. For `B`, `C`, and `S`, the current `meeting_position` call permits only the existing widening-position conversion, so a value presented as `int` is refused. This audit shows that the return position itself must be allowed to perform the implicit JVM narrowing even when there is no `i2b`, `i2c`, or `i2s` instruction in the body.

The smallest reuse point is the existing return-position `Cast`/primitive conversion presentation at the real `ireturn` consumer. It should be driven by the method descriptor and the int-shaped value's already-proven type, preserving the one value chain and the actual return anchor. It does not require a new pass or general local type inference. The `switch_returns` route must keep its existing distinction between the arm presentation position and the joining `ireturn` anchor; this straight-line audit does not authorize looking up an arbitrary `0xac` by the arm's `at` value.

`Z` is a separate boundary. The current builder intentionally requires boolean evidence (a `Z` parameter, `Z` call/field result, a proven boolean local, or a literal in a boolean context) and refuses an unproven int. The patched direct oracle demonstrates the JVM result is `value & 1`, but recovering that from an arbitrary int requires an explicit bitwise `& 1` expression or an equivalent proven boolean source. The existing bitwise work and boolean proof can be reused only where their facts already establish that expression; this audit does not justify general int-to-boolean range inference or a fabricated boolean cast.

The explicit primitive conversion opcode scope remains independent. `recover-primitive-conversions` covers actual `i2b/i2c/i2s` and the other conversion opcodes present in the bytecode. These four classes contain no conversion opcode at all: the narrowing is implicit in the `ireturn` descriptor. The return-position case must therefore be tracked separately from the 15 explicit conversion opcode families.
