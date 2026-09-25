## Context

当前混合字段图已用每个测试的真实 fallthrough/taken、canonical 精确前驱、SSA 测试依赖、常量 producer、唯一同槽 Phi 与唯一 `putstatic Z` 证明后发射一个字段赋值。冻结 `MixedLocalReturn.value(Z)Z` 的 BCI 0/1、4/7、10/13、16/17、20 与字段样例同型，BCI 21 改为 `ireturn`；普通 Region walk 重访 BCI 16，整体引用。调用者看不到返回值，但 source map 已保留所有十个已解码 BCI。

## Goals / Non-Goals

**Goals:** 只扩展已有闭合图的末端消费证明；直接返回 `Z` 时一次发射 `Return { value }`，保留惰性调用和值；拒绝时一个完整 owner 与逐指令来源；字段读写的现有行为不变。

**Non-Goals:** 任意布尔表达式全图重写、把 `ireturn I` 猜成布尔、支持局部存储后再返回、异常边跨图、非规范 JVM `Z` 值推断、改变字段规则报告或引入新公开 Region/AST。

## Decisions

1. **同一图，不复制 recognizer。** 保留私有 `ShortCircuitValue` 的有向无环测试/producer/consumer 归属，只把 consumer anchor 的准入从静态字段写入扩成真实 `ireturn`。两种情况都要在提交 owner 前拥有同一消费块且两个 producer 正常边唯一进入它；不能从 `return` 文本推断图边。若需要表示消费者种类，仅用最小私有证明形状而非新 pass/Region。
2. **返回描述符与 SSA 双重验证。** 原始方法描述符必须声明 `Z`，消费块首个真实指令为 `ireturn`，读恰好是同槽 Phi，Phi 两输入与生产者 1/0 一一对应且只有这一次使用；消费块无额外效果。每个测试的真实跳转目标、精确前驱、表达式值树与异常边拒绝复用字段图证明。`(Z)I` 等形状不能通过对 `ireturn` 的 opcode 单独判断返回类型。
3. **复用 Java AST、保持惰性与来源。** 沿已证明的边递归构造现有 `Conditional`，把整数 1/0 值在真实 `Z` 返回位置经已有布尔收窄路径转换，再写 `StmtKind::Return { value: Some(...) }`。共享后继的互斥文本复制仍受当前展开深度、大小与输出预算约束；测试、goto、producer、ireturn 都应在来源映射可追，后缀按原 Region 处理。
4. **拒绝先行。** 额外入口 `ChainExtraBoundary`、异常边、双消费者、独立效果及未证明返回类型仍整图拒绝，不能因为加了 return 消费就把旧 fallback 改成伪结构化 Java。字段正例、两/三测试纯链与当前混合字段双方法均作相邻回归。

## Risks / Trade-offs

- [Java boolean 返回可能把原始 JVM 整数归一化] → 只接受已证明的 `iconst_1/0` 两叶与 `Z` 描述符，非 0/1 或 `I` 返回拒绝；完整类 JVM 值/调用计数复验。
- [未消费的 producer 或外部入口被误吞] → 精确 canonical 前驱、SSA Phi 唯一 use、owner 原子提交，额外入口永久负例。
- [扩展字段证明导致回归] → 同一图与预算路径，字段两/三测试、混合 16 路径和异常拒绝完整复跑。
