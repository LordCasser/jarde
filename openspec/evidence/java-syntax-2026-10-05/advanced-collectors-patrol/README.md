# Stream 收集器进阶巡查（2026-10-06 root，第 154 前沿，负结果）

## 探针

[fixture/SC.java](fixture/SC.java)（`--release 8`）：三参 groupingBy（classifier+TreeMap::new 工厂+counting 下游）、下游 mapping+joining、partitioningBy、toMap 三参（键冲突合并函数）。

## 结果：健康（quotes=0）

- 两方法完整恢复：方法引用classifier/构造器引用工厂/双 lambda 下游全保留，cast 链如实；
- 往返编译 exit 0、`-Xverify:all` 行为 `2/1/BB,CC`、`2/2/2` 逐行一致。

## 处置

负结果归档；Collectors 全谱（基本→下游→工厂→合并函数）确认覆盖。
