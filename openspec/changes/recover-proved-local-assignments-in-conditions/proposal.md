## Why

[CF-06 固定对照](../../evidence/java-syntax-2026-09-27/cf06-inner-assignment/report.md)中，原 class 与固定 JADX 的完整 Java 8 源码七行运行一致；Jarde 对 `dup; istore/astore` 后同一值继续参加分支测试的两种方法均无法输出可编译的完整类。按现有语句方式拆出赋值会改变短路路径或丢掉赋值结果，必须先证明值的单次求取与赋值位置。

## What Changes

- 对物理 `dup; store` 且同值继续进入条件比较的局部赋值，在完整 SSA/CFG、类型和局部作用域证书下恢复源级赋值表达式，保留短路求值顺序。
- 仅让已证赋值写入在原可达分支执行；调用或字段读取产生的值不重复计算，局部声明与后续读取保持一致。
- 对共享消费者、额外用途、异常边、错类型或预算停止维持原子拒绝；用 CF-06 两个完整方法三方 Java 8 重编和验证运行验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证局部赋值作为条件表达式的值被恢复，保留短路语义与后续局部读取。

## Impact

影响 `crates/jarde-java` 的表达式 AST/打印优先级、现有条件值和短路值证明、局部声明/指令归属；优先复用原有 SSA、CFG、`Region`、来源和预算，不新增第二套区域图。JADX `IfCondition`、`IfRegionVisitor`、`ProcessVariables` 提供算法参考，但其是否内联赋值仍受本项目物理值/所有权证明约束。本任务不处理字段或数组**写入**表达式、CF-03 共享尾、CF-05 数值窄化及 CF-07 循环区域所有权。
