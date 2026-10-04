# Design：异型分支三元的语句化拆分

## Context（root 已实测）

- 拒绝点：条件表达式两分支值类型异构（`Integer` vs `String` 汇合 `Object`）时呈现层无法证明条件类型 → "the two values joined … do not have a conditional Java type this run can prove" 整方法拒。
- 同型分支全恢复；jadx 拆 if/return 有解。
- `IF.poly` 是 `return` 消费形（最易语句化）；实参嵌套形（`foo(c ? 1 : "s")`）语句化需要临时变量，本片**不做**（保持拒绝）。

## 决策 1：仅 return 消费形语句化（MVP 边界）

`return cond ? a : b;`（a/b 类型异构）拆为：

```java
if (cond) { return a; }
return b;
```

与 jadx 呈现同构。**其它消费形（赋值/实参/嵌套）保持既有拒绝**——它们的语句化需引入临时局部与顺序约束，超出 MVP 且有呈现漂移风险；登记为后续扩验。

## 决策 2：判据复用既有拒绝信息，不新建类型系统

拆分条件 = 既有拒绝已经成立的同一事实（"两分支无可证公共类型"）**且** 消费点恰为方法体尾部的单一 `return`。实现上：该拒绝发出处检查消费上下文，可拆则拆、不可拆保持拒绝——不新增任何 LUB 计算。

## 决策 3：零回退与负例

- 同型分支（int 链/嵌套/引用同型）三 元呈现逐字不变；
- 实参嵌套异构形仍拒（负例冻结）；
- 行为：`IF` 渲染源集 `javac` exit 0、输出 `4/9/true/2/s` 与原 class 一致（巡查已证占位等价路径，本片把占位变真实现）。

## 验证标准（可证伪）

1. 主锚 `poly` 恢复为 if/return 拆分形、整类 `javac --release 8` exit 0、行为一致；
2. 零回退：`dense`/`chain`/`io2`（同型三元）逐字节不变；负例（实参嵌套异构）仍拒；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + fingerprint。

## Open Questions

1. 拒绝发出处的具体锚点（task 1.1 定位：拒绝文本 "two values joined" 的代码位置与其对消费上下文的可见性）；
2. else 分支是否带 `else` 关键字（jadx 形无 else、尾 return 落方法体）——按仓库既有 if/else 呈现约定，实现者定并测试钉死。
