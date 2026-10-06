# java.util.function 组合子巡查（2026-10-06 root，第 152 前沿，负结果）

## 探针

[fixture/FC.java](fixture/FC.java)（`--release 8`）：Function.compose/andThen（组合序语义）、Predicate.and/or/negate 链、BiFunction/Supplier 闭包捕获。

## 结果：健康（quotes=0）

- 三方法完整恢复：lambda companion 形 + 组合调用保留（`doubleIt.compose(inc)` 序语义还原正确）；闭包捕获（`mk` 捕 `add`）正常；
- 往返编译 exit 0、`-Xverify:all` 行为 `12/11`、`true/true`、`v5` 逐行一致（组合序判别成立：compose=(x+1)×2=12 vs andThen=x×2+1=11）。

## 处置

负结果归档；java.util.function 组合域确认覆盖。java.time 簇的拒绝面（Temporal 族）与其正交。
