# Iterable 增强 for 异常边界探针

日期：2026-09-24。该证据只覆盖 direct `Iterable.iterator()`、`Iterator.hasNext()`、`Iterator.next()` 的投影边界；没有修改生产代码、OpenSpec tasks 或 roadmap。

## 结果

原始 Java 8 class 用 `javac --release 8` 分别以 `-g`、`-g:none` 编译，并由 `java -Xverify:all` 执行。两个原始 class 的 12 组结果相同，见 [`original-debug.txt`](original-debug.txt) 和 [`original-nodebug.txt`](original-nodebug.txt)。完整探针源码与 runner 在本目录；两版原始 class（含嵌套 iterator、异常类）分别保存在 `classes-debug/` 与 `classes-nodebug/`。

JADX 版本为 1.5.6。将完整 class 集合打包后分别反编译，再以 `javac --release 8` 编译 JADX 源码，并对重编译结果执行 `java -Xverify:all`。两种源码都能编译、验证，但都改变了异常处理结果：

| 场景 | 原始结果 | JADX `-g` 重编译 | JADX `-g:none` 重编译 | 边界差异 |
|---|---|---|---|---|
| `iteratorBoundary/hasNext` | `0;PIHLcatchF; caught=1; finally=1` | `-1;PIHIcatch; caught=1; finally=0` | `-1;PIHIcatch; caught=1; finally=0` | `hasNext()` 从 loop handler 落入只负责 `iterator()` 的 catch；finally 未执行。 |
| `iteratorBoundary/next` | `0;PIHNLcatchF; caught=1; finally=1` | `-1;PIHNIcatch; caught=1; finally=0` | `0;PIHNLcatchFHF; caught=1; finally=2` | `-g` 将 `next()` 移入 iterator catch；`-g:none` 虽落入 loop catch，却继续循环并重复执行 finally。探针让失败的 `next()` 先推进一次迭代器，避免错误重试导致挂起。 |
| `nextBoundary/hasNext` | `-99;PIHFX; caught=0; finally=1` | `-99;PIHX; caught=0; finally=0` | `-99;PIHX; caught=0; finally=0` | `hasNext()` 的异常离开了原 finally 覆盖区。 |
| `nextBoundary/next` | `0;PIHNNcatchF; caught=1; finally=1` | `-99;PIHNX; caught=0; finally=0` | `0;PIHNNcatchF; caught=1; finally=1` | `-g` 把 `next()` 放在元素体 catch 之外；`-g:none` 在此场景保留了异常处理。 |

表中的空格仅用于阅读；完整逐行文本保存在 `jadx-debug/rebuilt-output.txt` 与 `jadx-nodebug/rebuilt-output.txt`。探针还覆盖了统一 try/catch/finally 的正常路径和三种调用点各自抛异常；这部分原始与两份 JADX 重编译输出一致。JADX 的完整输出源码在两个 `jadx-*/IterableExceptionProbe.java` 文件，重编译 runner 与编译器输出也一并保留。

这些结果给出明确的 handler 边界：增强 for 隐式执行的 `iterator()`、每轮 `hasNext()`、`next()` 只能在它们进入新语法后仍落入相同处理器集合时替换原循环。调用前的可观察前缀必须留在 iterator 初始化之前；元素读取和 cast 也必须留在原 body 处理器范围内。探针中的 `iteratorBoundary` 和 `nextBoundary` 都表明，只比较循环节点或循环 body 的粗粒度 region 不足以证明安全。

## 对 Jarde 当前实现的判定

Jarde 完整 class-source（debug 与 no-debug）及单方法恢复结果已保存在 `jarde-debug/` 和 `jarde-nodebug/`。三个目标方法都明确标为 explanation-only，原因是 `local 1 crosses a quoted fallback region`；该文本缺少方法返回语句，`javac --release 8` 对完整类重编译失败（`jarde-*/javac.exit` 为 1，错误是三个方法缺少返回语句）。因此无法对 Jarde 重编译程序执行 JVM 行为对照，也不能据此宣称当前 Jarde 投影存在或不存在误投影。

可以确认当前候选实现的拒绝边界与本探针要验证的条件一致：[`build.rs`](../../../../crates/jarde-java/src/build.rs) 的 `iterable_for_each_candidate` 从 `hasNext` effect 取 handler 集合，并要求 `iterator()` 初始化、`next()`、调用指令以及可选 inline cast 的集合相同，否则返回拒绝。CLI `cargo check -p jarde-cli` 在独立 `/tmp/jarde-iterable-evidence-target` 下通过；任务结束后该 target 已由 `cargo clean` 清除（清理了约 1.2 GiB）。

## 复现命令

```sh
javac --release 8 -g -d /tmp/probe/debug IterableExceptionProbe.java
javac --release 8 -g:none -d /tmp/probe/nodebug IterableExceptionProbe.java
java -Xverify:all -cp /tmp/probe/debug ProbeRunner
java -Xverify:all -cp /tmp/probe/nodebug ProbeRunner

# 对完整 class 集合用 JADX 1.5.6 反编译；其输出源码与带 package defpackage 的 ProbeRunner.java 一起编译。
javac --release 8 -g -d /tmp/probe/rebuilt-debug /tmp/probe/jadx-debug/*.java
javac --release 8 -g:none -d /tmp/probe/rebuilt-nodebug /tmp/probe/jadx-nodebug/*.java
java -Xverify:all -cp /tmp/probe/rebuilt-debug defpackage.ProbeRunner
java -Xverify:all -cp /tmp/probe/rebuilt-nodebug defpackage.ProbeRunner
```
