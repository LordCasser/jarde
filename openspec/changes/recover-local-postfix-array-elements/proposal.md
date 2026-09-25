## Why

`new int[]{1, a++, a * 2}` 的 Java 8 字节码在两个数组 store 之间执行 `iload a; iinc a,1`。Jarde 把 `iinc` 当独立语句，未能闭合数组初始化链，完整类缺少返回；JADX 1.5.6 可编译且本例值正确，但其自身测试将原始后置自增语法的还原标为未完成。

## What Changes

- 在现有数组初始化证明中识别精确的局部旧值 load 与随后 `iinc slot,+1`，仅当二者同槽、同元素求值位置、SSA 消费和类型均闭合时，把它呈现为 `local++` 元素。
- 复用已有 `PostIncrement` 表达式及数组初始化的顺序、类型、异常、来源、预算与原子回退合同；下一元素读取更新后的局部值。
- 保留 `iinc +2`、不同槽、额外 use/入口或其他无法证明的副作用为普通语句/完整拒绝，不用算术近似掩盖语义。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：恢复数组初始化元素中的经证明局部后置自增值。

## Impact

主要修改 `jarde-java` 的现有数组初始化证明和 `PostIncrement` 的局部值构造，加入 Java 8 完整类及 verifier-valid 负例测试；不变更公开查询接口、CLI、Region 结构或全局局部变量重写。依赖一维数组初始化能力；与正在实施的多维嵌套数组变更共享 Builder 接缝，代码实施须串行。冻结三方证据与 `iinc +2` 控制见 `../../evidence/java-syntax-2026-09-25/array-postfix-element/analysis.md`。
