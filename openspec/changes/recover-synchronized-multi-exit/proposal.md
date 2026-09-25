## Why

合法 Java 8 `synchronized (lock) { if (flag) return first(); return second(); }` 有两条正常 `monitorexit` 和一条异常清理出口。现有 `guard::monitor` 仅接受一条正常出口，Jarde 对完整类保守引用六处并缺返回，无法编译；原 class 与 JADX 的三项正常/抛错执行相同。完整证据见 `../../evidence/java-syntax-2026-09-22/synchronized-multi-exit/`。

## What Changes

- 对一个锁、一处进入、两个由同一个条件分支通向不同返回的正常出口，以及共同异常处理器的有限形状，证明每条路径退出同一监视器恰好一次。
- 在受保护的 guard 正文内复用现有 `if`/返回表达式结构，保留两臂的值求取与异常时机，不把两条返回平铺或复制到同步块外。
- 失败时保持完整引用与真实来源；普通单出口 synchronized、try-with-resources、finally 与一般循环中的 break/continue 不被此 change 扩张。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对已证明两条正常返回和共享异常出口的同步块，输出行为等价的 Java `synchronized` 结构。

## Impact

影响 `jarde-java` guard 证明、受保护 body 的区域归属及现有 AST 构造；没有新 crate、依赖、公共 API 或目标代码执行。与 `recover-proved-finally-cleanup` 同属 guard/region 文件，生产实施按 root 的 Cargo 窗口串行，不混入当前复合 `+=` 修复。
