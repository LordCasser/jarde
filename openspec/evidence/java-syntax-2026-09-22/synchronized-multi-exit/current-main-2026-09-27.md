# CF-19 当前主线复核

本目录的 Java 8 冻结样本和完整三方基线见 [analysis.md](analysis.md)；`SynchronizedMultiExit.class` 的 SHA-256 仍为 `6658190aa7575f8be5095d71aa3ee07aa453d464666b1d34dea6a962950c45f2`。2026-09-27 在主线 `4ffff657` 上从临时 `CARGO_TARGET_DIR` 构建当前 `jarde-cli`，对该 class 执行：

```text
jarde-cli class-source --input SynchronizedMultiExit.class --class SynchronizedMultiExit --policy single-class --release 8 --format text
```

CLI 退出 0，但 `choose(Ljava/lang/Object;Z)I` 仍是 explanation-only：正文有 `// @bytecode 0 8 15 22` 和 `local 2 crosses a quoted fallback region`，没有 Java 返回语句。将未修改的完整 Jarde 类文本与冻结 runner 用 `javac --release 8 -g:none` 重编，`SynchronizedMultiExit.java:37` 报“缺少返回语句”，所以没有 Jarde 运行结果。旧审计中的六处字节码引用在当前主线缩为一处；这不构成语法恢复。

Root 同次独立从冻结源重编原类，class 哈希相同，`java -Xverify:all` 运行三行分别为 `first:return=10:trace=1`、`second:return=20:trace=2`、`first-throws:throw=java.lang.IllegalStateException:same=true:trace=1`。冻结的 JADX 完整类文本加同包 runner 同样 Java 8 重编和验证运行，三行逐字相同。由此 CF-19 的**多正常出口 synchronized 子形态**在当前主线仍是已证差距；单出口、嵌套锁和 synchronized 方法的整个单元尚未据此评分。实现任务沿用 [recover-synchronized-multi-exit](../../../changes/recover-synchronized-multi-exit/) 的独立 OpenSpec，不与成员/枚举改动混合。
