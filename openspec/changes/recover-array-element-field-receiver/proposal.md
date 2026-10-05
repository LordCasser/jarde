## Why

[array-element-field-receiver 巡查](../../evidence/java-syntax-2026-10-05/array-element-field-receiver-patrol/README.md)：`for(Item c : all) map.put(c.label, c)` 的循环体被 "field access … not one this run proved names the member its own receiver's type declares" 引注吞空，静态块其余部分幸存——**剥离编译后查找表静默为空（null/null/null vs 两对象+null，可编译错码）**。判别矩阵钉死根因：直接参数/局部别名字段读恢复；一切 **`aaload` 来源接收者**（for-each 元素、`xs[0]`、元素先入显式类型局部）失败；当前类/伴生类组件同败——元素值未携带数组组件类型，字段身份证明无从成立。jadx 完整解。

## Root 预审计（2026-10-06，读码）

- 拒绝发出点：build.rs:18997（`fields.claim(at)` 为 None）；字段身份证明在 field.rs:685 `verify`，接收者类型来自 **field.rs:786 `stated_type`——只读 SSA 命名引用**，aaload 结果在此通道无类型；
- **既有通道已能回答**：build.rs:25393 `array_of_value` 对 `Operation::ArrayElementLoad` 递归数组操作数并降一维（注释明言"a subscript reads the element the array's own type names"）；参数数组（SSA 命名 `[LItem;`）与 newarray 局部链都被它覆盖；
- field.rs:540 `plan` 的签名**已持有 `operations`**——最小修法是把该通道（直接调用或以闭包/小 trait 注入，避免 field→build 反向依赖）接入接收者类型判定；全局 SSA 类型注入是更宽的替代，非 MVP 首选。

## What Changes

- 字段身份证明的接收者类型判定接入**既有数组组件通道**（`array_of_value` 或等价注入）：`xs[i].field` / `for(T x : xs) x.field` 与直接参数字段读同判；不新建类型系统；
- 无诊断文本变更：本例的正确终态是**恢复**（无引注），不是新拒形。

## 硬不变量

1. 直接参数/局部别名字段读渲染逐字节不变（对照组零回退）；
2. 组件类型只能来自可证数组引用类型（方法描述符/字段描述符/已证 newarray）；不可证时保持现拒形——不得猜测；
3. 不触碰 DT-26 的 newarray 值通道（相邻但不同：这里是已有数组的元素加载）。

## 验收

- RG 主锚：剥离编译 exit 0、运行输出与原一致（两个 `Item@` hash 位 + `null`——以查找结果非 null/序为判据）；
- RM 边界零回退：数组元素**方法调用**（`xs[0].len()`/for-each `x.len()`）当前已恢复，变更后渲染与行为逐字节不变；
- RH.total `5`、RK/RL 直接元素形恢复（`q/q`、`s/i`）；
- EM enum 查找表循环体恢复（`B/A/null`；enum 常量池形债不在本片范围）；
- 直接参数/别名对照零回退；全门禁。
