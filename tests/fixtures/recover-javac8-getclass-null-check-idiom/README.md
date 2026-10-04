# fixtures: recover-javac8-getclass-null-check-idiom

**编译器版本（全部类文件的真实来源，这是本片的意义所在）**：

- `n1x/`、`g/`、`wrap/` 的 `.class`：**真 javac 8** — Corretto 1.8.0_432
  （`/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac`，
  `javac -version` → `javac 1.8.0_432`，**无 `--release`**，默认目标即 Java 8）。
  命令行（在各自目录内）：`<corretto8>/bin/javac -d . <Name>.java`。
- 同目录的 `.java` 是与 `.class` 对应的源（`g/G.java` 为巡查冻结原件的副本，
  SHA256 见证据目录 `SHA256SUMS`）。

**为什么必须用真 javac 8**：对 `outer.new Inner(…)`，javac 9+（含 `--release 8`）
发射 `invokestatic Objects.requireNonNull(Object)Object` 作为 null 检查，
真 javac 8 发射 `invokevirtual Object.getClass()Class`。本仓库既有验收锚全部是
javac 23 产物，对后者不可见（gap 的成因）。这些 fixture 冻结的是 getClass 拼写腿。

**字节码形态（n1x/N1x$Stat.use，javap -c）**：

```text
0: new           #2   // class N1x$Inner
3: dup
4: aload_1
5: dup
6: invokevirtual #3 // Method java/lang/Object.getClass:()Ljava/lang/Class;
9: pop
10: bipush        9
12: invokespecial #4 // Method N1x$Inner."<init>":(LN1x;I)V
15: invokevirtual #5 // Method N1x$Inner.total:()I
18: ireturn
```

**引用它们的 CI 测试**：`tests/recover_javac8_getclass_null_check_idiom.rs`
（正例 N1x/Wrap 双腿折叠、负例 G 用户 `getClass();` 语句保留、
`pop` 被替换的合成探针按既有拒绝码响亮拒绝）。
测试的 javac 23 腿由同一 `.java` 源以 `javac --release 8` 在测试内现编。

class 文件 SHA256 记录于
`openspec/evidence/java-syntax-2026-10-04/recover-javac8-getclass-null-check-idiom/SHA256SUMS`。
