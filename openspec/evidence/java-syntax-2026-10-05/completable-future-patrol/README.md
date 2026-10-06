# CompletableFuture 异步链巡查（2026-10-06 root，第 153 前沿，混合）

## 探针

[fixture/CF.java](fixture/CF.java)（`--release 8`）：supplyAsync/thenApply×2/String::toUpperCase 方法引用链、exceptionally 恢复链（抛异常 supplier）、completedFuture+thenCombine 合流。

## 结果

- **chain/recover 完整恢复**（lambda 内联+方法引用包装+`exceptionally(e->…)` 泛型 SAM 链）；往返（companion 名内联后）`-Xverify:all` 行为 `DATA!` / `fallback:java.lang.IllegalStateException: x` 逐行一致；
- **combined = 第 6 族表位新行**：`thenCombine` 首参 `CompletableFuture presents ... requires java.util.concurrent.CompletionStage`——javadoc `CompletableFuture implements CompletionStage<T>, Future<T>`（单行表族）；幸存缺 return=SAFE；jadx 同构可解。

## 处置

`CompletableFuture→CompletionStage/Future` 行并入 [recover-temporal-argument-widening](../../../changes/recover-temporal-argument-widening/proposal.md) 的行集（同机制同落点，表批 2）。
