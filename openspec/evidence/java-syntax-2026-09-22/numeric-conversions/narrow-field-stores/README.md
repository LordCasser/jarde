# Narrow integer field-store audit

This source-only audit constructs verifier-valid Java 8 bytecode that Java source cannot express directly: eight public `int` fields are compiled with ordinary `putfield`/`putstatic`, then the class file patch changes each selected `field_info` descriptor and the matching `Fieldref` `NameAndType` descriptor to `B`, `C`, `S`, or `Z`. Every selected field gets its own appended `Utf8` constant. Method descriptors and Code bytes are not rewritten. `NarrowFieldStoreEffects.value` supplies a normal or throwing producer, and the `set*On` helpers place a nullable objectref on the operand stack so producer-before-null behavior is observable inside the method rather than being intercepted by Java method invocation.

冻结 CLI 为 `/tmp/jarde-cli-narrow-field-stores-48ed`，运行前后 SHA-256 均为 `48edb9d2e3eec451983aabb4affcbcaf6d723c8a75b0284605f729743088cc76`。`run_audit.py` 在自动删除的临时目录中重放完整审计：执行 `javac --release 8 -g:none`、精确 class 补丁、`javap`、`java -Xverify:all`、冻结 CLI 和 JADX，并将源码、class 快照、报告、运行输出、状态与哈希保存于此。本次不运行 Cargo。

## Exact class patch

The main class has 16 fields and 49 `Code` methods including the constructor. The eight selected fields begin as `int` and are patched as follows:

| field | field descriptor | `Fieldref` operation | selected writers |
| --- | --- | --- | --- |
| `byteField` | `I` → `B` | `putfield` | `setByte`, `setByteProduced`, `setByteOn`, `setByteProducedOn` |
| `charField` | `I` → `C` | `putfield` | `setChar`, `setCharProduced`, `setCharOn`, `setCharProducedOn` |
| `shortField` | `I` → `S` | `putfield` | `setShort`, `setShortProduced`, `setShortOn`, `setShortProducedOn` |
| `booleanField` | `I` → `Z` | `putfield` | `setBoolean`, `setBooleanProduced`, `setBooleanOn`, `setBooleanProducedOn` |
| `staticByteField` | `I` → `B` | `putstatic` | `setStaticByte`, `setStaticByteProduced` |
| `staticCharField` | `I` → `C` | `putstatic` | `setStaticChar`, `setStaticCharProduced` |
| `staticShortField` | `I` → `S` | `putstatic` | `setStaticShort`, `setStaticShortProduced` |
| `staticBooleanField` | `I` → `Z` | `putstatic` | `setStaticBoolean`, `setStaticBooleanProduced` |

`patch-report.json` records each `field_info` offset, independent descriptor constant, exact `Fieldref` and `NameAndType` indexes, all writer methods, and every method's Code hash before and after. All 49 Code hashes are unchanged. The source class SHA-256 is `ae0bfc789051a30dd6779d57cea2f484d7e5fe4822f18102eb74915075d04eda`; the patched class SHA-256 is `713e821a6e182505d412ae2b8b01823e426cd436ac154073044fc19f1c6d2f10`. Both `javap` listings contain 28 `putfield`, 20 `putstatic`, and zero `i2b`, `i2c`, or `i2s` instructions.

## JVM behavior

The runner uses 21 boundary and overflow values. It records 380 lines: 88 instance direct cases, 96 instance producer cases, 84 static direct cases, 88 static producer cases, and 24 ordinary controls. Original and patched classes both pass `java -Xverify:all` with status 0. The complete outputs are `original-runtime.txt` and `patched-runtime.txt`.

The patched direct and normal producer stores narrow exactly as the JVM field operation does: byte and short values wrap at their widths, char values retain the low 16 bits as an unsigned value, and boolean values follow the observed low-bit behavior. Failed producers leave the prior field value unchanged and increment the producer count once. `set*On(null, value)` throws `NullPointerException`; `set*ProducedOn(null, value, true)` throws the producer's `IllegalStateException` with one call; `set*ProducedOn(null, value, false)` calls the producer once and then throws `NullPointerException` at `putfield`. This preserves the producer-before-null ordering inside the actual field-store instruction.

The ordinary controls are deliberately split into one method per write: each type has a local-variable write, a zero/false constant write, and a one/true constant write, for both instance and static fields. The runner reads the field after each method, so the first local write is independently observed rather than being hidden by a later write. Their patched output is the 24 `ordinary-*` lines at the end of each type's section.

The Java SE 8 JVMS states that `putfield` and `putstatic` accept an `int` stack value for fields declared `boolean`, `byte`, `char`, `short`, or `int`, then apply value-set conversion; it also defines the null receiver behavior for `putfield`. See [§6.5 putfield](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html#jvms-6.5.putfield) and [§6.5 putstatic](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html#jvms-6.5.putstatic).

## Recovery comparison

`jarde.java.txt` and `jarde-report.txt` preserve the complete frozen-CLI class-source result. The CLI exits 0. Its `field_write` meeting position refuses every selected int-to-narrow field write, emits a refusal comment, and substitutes `return;`. The complete Jarde source still compiles and runs (`jarde-javac.status` 0, `jarde-runtime.status` 0), but the run is semantically wrong: fields retain their initial values, producer calls stay at zero, and null receiver cases return. This is an explicit recovery refusal that becomes a no-op in the compilable whole-class text, rather than a correct implicit narrowing.

`jadx.java.txt` is the unchanged JADX class source. Its full compile fails at 24 int-to-byte/char/short/boolean assignments (`jadx-javac.status` 1), so no JADX runtime result is claimed. The package-prefixed helper and runner used for that compile live only in the temporary replay directory.
