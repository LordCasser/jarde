# 条件物化 RHS 的字段复合恢复（recover-conditional-rhs-field-compound）

## Why

[boolean-loop-earlyret 巡查 + root 2026-10-07 亲测复现]：BI 锚 15（已登记第 15 critical 锚）——`this.ok &= x > 0`（receiver copy 跨 getfield+putfield + **条件物化 RHS**，`x > 0` 经 `if_icmpXX` 双臂常量物化）在幸存循环体内被引注吞掉，整类剥离编译后 `earlyRet` 答 `true` vs 原 `false`（`false/false/false/true` vs `false/false/false/false`，冻结 jar `bi.jar` root 实测）。`recover-chained-field-assignment` 已交付 FieldCopies Receiver 形状与复合 op 臂，但其条件物化 RHS 边界（跨块）未开——该边界一开锚 15 即恢复，类级可编译错面关闭。**恢复优于守卫扩展**：守卫的结构条件（幸存结构内引注）扩展是 fallback，本片成功则不需要。

## What Changes

- FieldCopies 的复合赋值 RHS 判据接受**已证条件值物化**（跨块）：`prove_conditional_value` 只读复用（inline-concat 同模式——证书按 CFG 陈述 region 交其重校验，无重实现），RHS 呈现为 `x > 0` 等源码形；
- 循环体内形：复合赋值语句位于幸存循环体（BI 形）不改变判据——receiver copy 的消费几何（一读一写）与条件 RHS 的物化几何各自独立证明；
- 负例：物化臂含副作用/多比较/异常边保持拒绝逐字。

## 硬不变量

1. 既有 FieldCopies 锚（CH 链/SC 累积/BF 复合）渲染逐字节不变；
2. 条件求值语义精确（`>` 两操作数各求值一次、不短路）；
3. 不得产出"可编译且行为不同"文本——**本片的验收即含 BI 整类剥离输出与原逐字一致**（`false/false/false/false`）。

## 验收

- BI.earlyRet 恢复（0 引注），整类剥离编译 exit 0、BI 自带 main 输出与原 class **逐字一致**（双腿）；
- 锚 15 关闭后 boolean-loop-earlyret 巡查的该 critical 行关闭；
- 全门禁 + oracle ignored 腿 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：条件物化 RHS 的字段复合赋值按源码形态呈现，循环体内形行为完整。
