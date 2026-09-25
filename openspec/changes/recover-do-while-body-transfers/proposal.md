## Why

`do-while` 本体已可恢复，但循环体首块含条件分支时，jarde 把它误当第二个循环测试；跳到闩锁或唯一出口的体内路径因此整段引用，完整类留下缺失的返回语句。已冻结的 Java 8 class 中，原 class 与可编译的 JADX 源码逐项一致，问题可限定在区域所有权与循环内转移，而不是扩大字节码解码。

## What Changes

- 证明头块的两条分支都留在循环体内时，把它作为体内分支，而非拒绝为“双测试”；到达唯一闩锁的路径可写成等价的单臂 `if`，不要求凭字节码猜原文是否写了 `continue`。
- 对体内显式转移到该层循环唯一出口的路径，只有在目标、所属层次及被跳过的语句都已证明时，写出 `break` 并让循环后的返回语句归属一次。
- 保留现有 `DoWhile`、来源、预算、取消及不能证明时的完整引用；带副作用的条件和普通循环保持现有执行语义。多层标记转移、`switch` 中的 `break`、任意跳转与 `for` 推断单列债务。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证明的 `do-while` 体内汇合与本层提前退出保持可编译的结构及原执行顺序。

## Impact

主要影响 jarde-java 的 loop/if 区域证明与最小控制转移 AST/发射；无需改 reader、SSA、依赖或新增通用 CFG 求解器。证据在 `../../evidence/java-syntax-2026-09-22/do-while/`。更宽的循环控制合同已记于 `present-proved-java-structure`，本项只实现该合同的单层 `do-while` 子集。
