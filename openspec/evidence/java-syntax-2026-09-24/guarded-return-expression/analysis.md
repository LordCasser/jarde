# 已证明 synchronized return 的值放置边界

## 样本与证明条件

基线是 `Locked.locked()I`，由 `javac 23.0.1 --release 8 -g:none` 编译。输入 class 为 285 字节，SHA-256 `9a38d88a79ceafbf31b14f68d028ead9a50b77445ce90488a33e10da53b79c73`。重编 class 的哈希完全相同。字节码是 `aload_0; dup; astore_1; monitorenter; aload_0; getfield n; aload_1; monitorexit; ireturn`；catch-all handler 在 BCI 11 以同一个锁槽释放 monitor 后重抛。

修复前的 `p3_sync_return` 缺陷形态是把 `this.n` 先写入合成 `saved0`，再在 synchronized body 内返回它：`int saved0 = this.n; return saved0;`。`has_independent_boundary` 把 BCI 8 的结构性正常 `monitorexit` 当作 producer 与 return 间的独立效果。当前实现只从一个已证明的 `Shape::Monitor` 取得三个精确事实：它关联的 return BCI、normal-exit BCI 和 body 范围。只有 final consumer 等于该 return、producer 落在该 body，且该唯一 normal exit 严格位于 producer 与 return 之间，才跳过这一条指令。handler exit 不会被选中；普通同步块、其它 consumer、body 外 producer、重复的 return 所有权和没有 Guard proof 的退出没有豁免。

因此这个例子的最终成员为 `synchronized (this) { return this.n; }`。source map 保留字段读取及 Guard 事实；已有测试验证包括 BCI 5 的字段生产、BCI 8 的 normal exit、BCI 10 的 return 和 handler BCI 11/13。`essential` 与 `all` class-source 报告的成员正文逐项相同。Jarde 生成的 `Locked.java` 通过 Java 8 重编；结果 class 与原 class 字节逐字相同，`java -Xverify:all` 输出 `locked:7`、`locked:-4`。JADX 1.5.6 的源码也可重编并得到相同两行。

## 反例与安全拒绝

`GuardReturnEffects.effect` 在持有 monitor 时先读旧值，再调用有计数、会修改字段且可抛异常的 `afterRead`，最后返回旧值。它是 javac 生成并由 `-Xverify:all` 接受的 Java 8 class。原 class 与 JADX 重编类都输出：

```text
effect:5:15:1
effect-throw:after-read:17:1
effect-lock:released
nested:3:13:1
nested-throw:after-read:14:1
nested-lock:released
```

这里值确实在独立效果前被读出；值、字段新值、一次调用、同一个异常及释放后的锁可用性都被 runner 观察。Guard 的 body proof 不接受这些独立语句，Jarde 为 `effect` 保留 `@bytecode 0 21 24 28` fallback；嵌套 monitor 样本同样保留 `@bytecode 0 28 31 38 45`。这说明新的退出豁免没有扩成“忽略所有 monitorexit”，也没有为了直接表达式把调用删掉或挪到读取之前。该拒绝结果是有意的安全边界，尚不声称 Jarde 对这两个反例完成源码恢复。

## 重现命令

```sh
javac --release 8 -g:none -d /tmp/locked-original tests/fixtures/p3-sync-return/Locked.java openspec/evidence/java-syntax-2026-09-24/guarded-return-expression/LockedRunner.java
java -Xverify:all -cp /tmp/locked-original LockedRunner
/opt/homebrew/bin/jadx --no-res -d /tmp/locked-jadx tests/fixtures/p3-sync-return/v8/Locked.class
jarde-cli class-source --input tests/fixtures/p3-sync-return/v8/Locked.class --class Locked --policy single-class
javac --release 8 -g:none -d /tmp/locked-jarde openspec/evidence/java-syntax-2026-09-24/guarded-return-expression/jarde-compiled/Locked.java openspec/evidence/java-syntax-2026-09-24/guarded-return-expression/LockedRunner.java
java -Xverify:all -cp /tmp/locked-jarde LockedRunner
javac --release 8 -g:none -d /tmp/effects tests/fixtures/preserve-guarded-return-expression/GuardReturnEffects.java tests/fixtures/preserve-guarded-return-expression/GuardReturnEffectsRunner.java
java -Xverify:all -cp /tmp/effects GuardReturnEffectsRunner
```

冻结 class、完整 `javap -c -v`、JADX/Jarde 正文、运行输出与 JSON evidence 均在本目录或 change fixture 目录；上表中的哈希可用 `shasum -a 256` 重算。
