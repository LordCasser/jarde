## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/varargs-ctor-arg-patrol/README.md)：W3.viaArrays 字节码 = 外层 `new ArrayList@0; dup@3; iconst_3@4; anewarray@5; [dup; index; valueOf; aastore]×3; invokespecial ArrayList.<init>(Collection)@…`。裸位判据：W4 显示既有 varargs 内联数组证明（调用位 `new Integer[]{…}` 呈现）已建——**第一个取证义务**：定位该判据（grep `asList`/varargs/内联数组 initialiser 的证明与呈现，`build.rs`/`init.rs`），确认其与构造走查的复用接口（可能需把"数组链证明"提取为可从走查调用的形态）。

## Goals / Non-Goals

**Goals:** 构造实参位数组链恢复（W3 两形、W1.use 全链）；裸位/嵌套构造位/普通调用位逐字不变。**Non-Goals:** 数组链喂**方法**实参（已健康不碰）；多维数组链；数组链双用途/跨块（拒绝）；`Arrays.asList` 之外的泛型桥语义（签名投影既有边界）。

## Decisions

1. **判据同源**：走查遇 `anewarray` 起始的完整链时，复用既有 varargs 内联数组证明（元素生产判据、区间封闭、单用途）；成功则记录实参为数组初始器表达式并跳过区间，外层 ctor 选到真正属于自己的 `<init>`；失败回退现拒绝文本。
2. **呈现复用**：`new T[]{…}` 初始器拼写（裸位同一呈现函数），无第三份。
3. **验收锚定**：W3（`3`/`0`）、W1（`6.0:7:W1`）+ 变体（空 varargs、混合装箱、`new HashSet<>(Arrays.asList(…))`）；负例（链中插语句、数组双用途）保持拒绝。

## Risks / Trade-offs

- **区间误跳** → 链完整连续 + 值单用途判据；插语句负例钉死。
- **与嵌套构造递归交叠** → 数组链与嵌套构造是不同形态分支，互斥识别；测试双向。
