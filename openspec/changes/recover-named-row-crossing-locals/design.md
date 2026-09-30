## Context

[巡查证据](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/README.md)固定了失败链与实验方向。`resources()` 门槛现状：无前置指令→跳过；handler 绑定 store→跳过；具名行前字段赋值→跳过；普通赋值 store（`copied_local` 失败）→跳过；非 store 语句（`statement_ends`+`statement_boundary`）→跳过；**其余 store（构造 store、调用 store）→完整 TWR 证明→`jre_guard_handler` 拒绝→阻断 catches**。C4 实证：真 TWR+具名 catch 的具名行 `[0,36)→39` 覆盖初始化、TWR 自身行为 catch-all，故"具名行起点紧跟完成 store"结构上排除 TWR。声明层：`all_reads_reach_presented_writes` 对每个写值过 `presented_int_store_value`（`Value::Int` 白名单：字面量/同块 load/静态调用/加法树），构造 store 失败即 `Incomplete`；finally 家族以 `nullable_resource_lead || … || null_lead_straight` 逐证书旗标绕行，本切片不加旗标而扩写值形状。

## Goals / Non-Goals

**Goals:** (1) 门槛的具名行-完成-store 判别（`statement_boundary` 即含"无栈残留、语句完整结束"的完成证明）；(2) 写值接受集新增"同块单用途已证构造点"（`new`/dup/init 三元组恰在 store 前连续、结果仅此一处消费——`constructed_initialisation` 同款纪律）；(3) N1/P3StorePrefix 翻转为正例并同步证据。

**Non-Goals:** catch-all 行一切降级；行劈开语句/吞初始化/交叠呈现负边界；`twrNamed`（TWR+外层具名，CF-17）；调用 store 之外的多语句 lead；finally 家族既有旗标语义变更；DEX。

## Decisions

1. **判别只加两个条件**：`row.catch_type_index.is_some()` + `statement_boundary(facts, before, row.start_bci)`。不要求 store 值来源分类（构造/调用同样安全——结构论证不依赖值来源）。放行后由 `catches` 的既有证明（保护区间、handler 序列、join）把关。
2. **写值扩展复用构造纪律**：接受"store 的读值是同块 `new`+dup+init 三元组的输出、构造指令连续、结果单用途、store 后无残留"作为可呈现写值；声明呈现为 `T local = new T(…);`（new-site 呈现）。不引入跨块构造或共享构造。
3. **翻转负例的证据义务**：N1/P3StorePrefix 在新旧二进制上的输出对照 + 三方行为（原 class 语义本就是普通 catch）+ TWR 家族（P5TwoResources、Test9 可空、多资源）不回退，写进 change 的验证记录。
4. **验收分层**：C1（five+swallow）、C4.constructNamed 完整恢复且 Java 8 重编运行等价；C3 对照组逐字不变；`twrNamed` 输出与基线二进制一致（仍拒绝）。

## Risks / Trade-offs

- **误把真 TWR 的具名包装行放行** → 该行起点在初始化之前（`before` 非 store），判别不命中；以 twrNamed 与 TWR 家族回归钉死。
- **构造写值呈现丢副作用序** → 三元组连续 + 单用途 + store 紧随，与 `constructed_initialisation` 同一不变量；"构造后插入语句"负例拒绝。
- **N1 翻转掩盖未来真 TWR 退化** → catch-all 降级一字不动，TWR 自身行永不被此判别触碰；巡逻证据记录论证。
