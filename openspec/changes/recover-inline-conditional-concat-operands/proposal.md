# 内联条件值作拼接链操作数（recover-inline-conditional-concat-operands）

## Why

[multiconsumer 停手取证](../recover-committed-local-multireads/CLOSED-premise-falsified.md)（双腿决定性探针 NI/NI2/NMA/NMB/CMP，冻结于其 `evidence/`）：`"" + x + (a == b) + y` 形——**内联**分支值（引用等值 `if_acmpXX` 双臂 iconst_0/1 物化；探针亦含调用链内比较）直接作 `+` 链操作数时，链因分支跨块，`concat@1` 的 `jre_concat_split`（"the concatenation that starts at BCI N ends in the toString at BCI M, which is in another block…"）拒整链 → 方法幸存但表达式整条丢失。判别完整：去分支形（NMA）/仅去比较（NI2）**完整恢复**；加一分支（NMB）逐字复现拒绝。这是日志/消息字符串的高频写法（`"" + a + (x == y) + b`）。

**邻接已闭片的边界**：`recover-ref-eq-boolean-argument`（7/7）覆盖比较值在 `append(Z)` **实参位**的布尔呈现；`recover-scv-concat-consumers`（7/7）覆盖**存储后**由拼接消费（`boolean f = …; return f + ":"`）；`recover-conditional-values` 覆盖两臂值汇合的通用证明。缺的是 **concat@1 链所有权证明接受"中间块恰为已证条件值物化"的跨块 toString**。

## What Changes

- `concat@1` 的链所有权/终端证明：链中间的块若**恰好**构成一个已证条件值物化（两臂各 iconst 常量、唯一 join、无其它副作用——复用 `recover-conditional-values` 的两臂证明作子证明），则跨块 toString 可拥有该链，比较值按既有布尔通道进 `append` 实参位。
- 零新表达式通道：条件值呈现与 append 实参位布尔呈现都是既有能力，本片只接链所有权判据；
- 负例：分支臂含副作用/多比较/嵌套拼接跨异常表的形保持 `jre_concat_split` 拒绝逐字不变。

## 硬不变量

1. 同链无分支形（NMA 类）与存储后消费形渲染逐字节不变；
2. `jre_concat_split` 的其它拒绝面（真跨块副作用链）零触碰；
3. 条件值求值语义精确（== 不短路，两操作数各求值一次——与 && / || 域区分）。

## 验收

- NI/NMB/CMP 恢复（0 `jre_concat_split`），剥离编译 exit 0、`-Xverify:all` 输出与原一致（NI `3/10/5/true`）；NMA/NI2 零回退；
- 门控实验先行：只接链所有权判据，NI 翻转、真跨块副作用负例不翻；
- 全门禁 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：内联条件值作拼接链操作数时链按源码形态呈现，方法行为完整。
