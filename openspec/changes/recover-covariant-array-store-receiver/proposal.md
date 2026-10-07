# 协变数组存入的接收者宽化（recover-covariant-array-store-receiver）

## Why

[array-covariant-store 巡查](../../evidence/java-syntax-2026-10-05/array-covariant-store-patrol/README.md)：`Object[] a = new String[2]; a[0] = Integer.valueOf(1);`（协变数组 + 异型存入）——无 LVT 时字节码只有 `anewarray` 组件型与 `aastore`，局部被呈现为分配组件型 `String[]`，异型存入文本 `String[]` 接收 `Integer` **剥离编译失败**（响亮安全形，第一不变量不违反；jadx 同败——其 `Multi-variable type inference failed`）。运行时 `ArrayStoreException` 行为面精确。**MVP（巡查已设计，无需 LUB）**：存入点在值类型与组件不兼容时，把存入的数组访问接收者宽化为 `((Object[]) local)[idx] = value;`——任何引用数组可上转 `Object[]`，源编译恢复且运行时 ASE 语义保留（不重建源声明型）。

**注意（2026-10-07 立项）**：本片登记后 main 已合入多个相邻片（`recover-array-element-field-receiver` 的组件类型证明、窄局部类型系列）——开工时**必须先复验巡查锚在 HEAD 的现状**（是否已被相邻片改变、拒绝文本是否漂移），以复验为准。

## What Changes

- 数组元素**存入**呈现的类型检查：当被呈现的接收者组件型与值类型不兼容（协变存入形）且接收者是引用数组时，存入点按 `((Object[]) local)[idx] = value` 宽化呈现；
- 判据复用 `array_of_value` 既有通道陈述的组件事实（与 `recover-array-element-field-receiver` 的读取侧同源、存入侧对称）；
- 同型存入与兼容存入零改动；非引用数组（基本型）不适用（无协变）；
- 不重建源声明型（不做 LUB/不猜 `Object[]` 是原声明）。

## 硬不变量

1. 同型/兼容存入锚渲染逐字节不变；
2. 运行时 ASE 语义保留（行为对照必测：存入后读回 + 捕获 ASE 的对照驱动）;
3. 不得产出"可编译且行为不同"文本（宽化不改变求值序——接收者求值一次）。

## 验收

- 巡查 AS 锚（storeWrong/storeNumber）恢复可编译：剥离编译 exit 0、`-Xverify:all` 运行 ASE1/ASE2 精确触发与原类一致（对照驱动记录）；同型对照零回退；
- 门控实验先行；全门禁 + oracle ignored 腿 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：协变数组的异型存入按宽化接收者呈现，编译性与运行时检查语义完整。
