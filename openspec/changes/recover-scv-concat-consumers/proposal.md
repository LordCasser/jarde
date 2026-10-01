## Why

[EM-19 位运算巡查](../../evidence/java-syntax-2026-10-01/em19-bitops-patrol/README.md)确认：位运算核心（long/复合赋值/位掩码条件/直接 return 短路链）全部健康；唯一缺口是**短路链存布尔局部后由拼接消费**（`boolean hasA = (v&1)!=0 && …; return hasA + ":";`）——`recover-short-circuit-local-values` 的消费方集合不含拼接链实参位，错误信息即该切片自身边界（B5.s1 固定复现，判别链完整：直接 return ✓ / 存+return ✓ / 存+拼接 ✗）。该形态（布尔标志进日志/消息字符串）是真实代码高频写法。

## What Changes

- 短路值切片的消费方集合扩展：值经拼接链的 `StringBuilder.append:(Z)` 或装箱 `append:(Ljava/lang/Object;)`（布尔装箱）消费时，按既有布尔呈现（无装箱 cast 拼写差异按既有通道处理）。
- B5.s1/s2、B4.v1–v3、B2.compound 全部完整恢复且行为一致；B3（直接 return）与 s3（存+return）逐字不变（diff 断言）。
- 值流断裂/共享消费者未证的既有负例语义不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：存局部的短路布尔值可作为拼接链实参呈现，恢复文本可重编。

## Impact

仅 `crates/jarde-java` 私有短路值切片（消费方枚举处）与呈现及测试；既有切片消费方语义零回退。不新增机制。
