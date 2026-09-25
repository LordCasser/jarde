# String switch: BMP and supplementary Unicode

This fixture freezes Java 8 behavior for the empty string, BMP Chinese, a supplementary character, default, and null. The `selector` increments a static counter so its call count is observable. Sources, original class, JADX/Jarde decompilation, tool identity, and hashes are stored in this directory. The original class was compiled locally with `javac --release 8 -g:none`; `javap.txt` records its bytecode.

## Frozen input and results

- `StringSwitchUnicode.class` SHA-256: `434e6e84a045d67bc0a6337d89f5493779e12dc01133d89355220ff57cf367fa`
- Source SHA-256: `a8315f5647b656eae17e9683b2a021a26aa0f253ccd6fd04b5a1190ed00ccc85`
- Compiler: local `javac --release 8 -g:none` (JDK 23.0.1; only obsolete-option warnings)
- JADX: 1.5.6; complete class at `jadx/StringSwitchUnicode.java`
- Jarde: `/tmp/jarde-catch-binding-replay/jarde-cli`, SHA-256 `95362354d3de2af1ac57f69ea9f7492731ca580afd2e5f7e3b16fc4144b3aa8c`

The original class under `java -Xverify:all` prints:

```text
empty:empty:calls=1
bmp:bmp:calls=1
supplementary:supplementary:calls=1
default:default:calls=1
null:NullPointerException:calls=1
```

JADX's complete decompiled class compiles with `javac --release 8 -g:none` and matches the original line for line under `-Xverify:all`; `jadx-diff.txt` is empty. Jarde's class-source text has only an unrecovered explanatory comment in `main`. It remains syntactically compilable, but the resulting class prints nothing; `jarde-diff.txt` records the original output against that empty output. Jarde's `choose` method itself recovers as compilable two-level integer dispatch.

## UTF-16 hash and mapping

Java `String.hashCode()` iterates UTF-16 code units using `h = 31*h + unit`. The empty string hashes to `0`; `雪` (U+96EA) hashes to `38634`. `𐐷` (U+10437) is the UTF-16 surrogate pair `D801 DC37`, decimal `55297, 56375`, so its hash is `55297*31 + 56375 = 1770582`. In `javap.txt`, the first `lookupswitch` uses buckets `0`, `38634`, and `1770582`; each bucket compares with `String.equals` and writes discriminator `0`, `1`, or `2`. The second `tableswitch` maps these to the three results and maps default to `"default"`. The selector is called once before hash dispatch.

The supplementary character stays intact in the constant pool and switch literal. Both decompiled sources also preserve `𐐷` directly, with no damaged surrogate pair or literal escaping error. This fixture validates UTF-16 bucket values and the two-level mapping, but does not cover special control characters that require `\\uXXXX` source spelling.

JADX folds the dispatch into one `switch (selector(str))`; this complete class compiles and runs equivalently. Jarde retains the selector's single evaluation, hash/equals checks, and second integer switch, with no label mismatch, but its unrecovered `main` makes the complete class behavior differ. Local compilability or correctness of `choose` is not complete-class three-way acceptance.

Root separately replayed the **recovered `choose` method** with
`StringSwitchUnicodeChooseRunner.java`: the external driver uses reflection only
to reset/read the private `calls` field and calls the public `choose` for the
same five inputs. It was compiled independently against the frozen original,
the default-package-only adjusted JADX class, and the unmodified Jarde class
source, all with `javac --release 8 -g:none`; all three executions used
`java -Xverify:all`. `choose-original.txt`, `choose-jadx.txt` and
`choose-jarde.txt` are byte-for-byte equal. Thus Jarde's **method-level**
UTF-16 dispatch and selector effects are proved for these inputs, while the
unrecovered `main` remains a distinct full-class gap. Neither this evidence
nor the correct two-stage output satisfies the change's single-string-switch
presentation requirement.

## 2026-09-24 后续：字段前缀 catch 修复后的完整类重放

[结构变更 2.4b](../../../changes/present-proved-java-structure/tasks.md)仅修改普通 `putstatic` 后的资源头分类后，root 用保留 CLI `/tmp/jarde-instanceof-replay/jarde-cli`（SHA-256 `336fda92b93df11a33299cb015df7b925b4ade22970727a21ff09b4b32156b71`）对**同一冻结 class** 重跑。`main` 现完整呈现 `calls = 0; try { choose((String) null); ... } catch (RuntimeException ...) { ... }`，全部五个方法均无 `@bytecode`。原源码原样重编 class 与冻结输入逐字节相同；JADX 输出仅去掉自动附加的 `package defpackage;`，原/JADX/Jarde 三份完整类均以 `javac --release 8 -g:none` 编译并用 `java -Xverify:all` 执行，五行输出逐字节相同（SHA-256 `41489c44ab3e0c532afb12365f3b396f19a5e6ecf809a1233feaf01e14750a95`）。

上面旧 CLI 的完整类失败记录仍保留，指向修复前的真实缺口；新结果只将此样本的**行为**闭合。Jarde 的 `choose` 仍是两级整数分派，没有输出 `switch(String)`，故不能据此勾选 `recover-string-switch` 1.2 或 2.x。
