# 嵌套泛型类头投影巡查（2026-10-03）

泛型 Signature 投影域定向取证（主线 `341c8309`）。固定转录 [fixture](fixture/)（Z1 泛型类 + 嵌套 `Box<U>`；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 orig.out（`b`/`[1, 1]`/`x`）。

## 现状矩阵（既有 10+ 泛型切片之上的前沿）

| 场景 | 主线 Jarde |
| --- | --- |
| 顶层泛型类头（`class Z1<T extends Comparable<T>>` 含 bound） | **已呈现** |
| 嵌套泛型类 `Z1$Box<U>` 类头 | `class_generic_source_unproved`（"class kind, nesting, name, or type-use annotations lack a faithful generic header position"） |
| 嵌套类成员的 U 域（`U value` 字段） | `jvm_signature_scope_unproved`——**链式失效**（头未投影 → U 不在可用域） |
| 嵌套泛型类折叠进外围 | 拒——成员类折叠片的 `Signature` 防线（子类带 Signature 不折叠）；**三重阻断闭环** |
| 同类泛型方法/字段体投影（`List<T>` items、`map` 方法等） | `generic_call_binding_unproved`（同类 Methodref 命中本方法或邻近重载，8×）/ `field_generic_body_unproved`（同类 Fieldref，4×）——呈现退化为原始类型（可编、非忠实） |

## 定性

四子族中 **`class_generic_source_unproved` 是链头**：嵌套泛型类头投影解锁 (a) 成员 U 域（scope 链）、 折叠（Signature 防线在头投影后按既有防御语义重新评估——若头可投影则防线不再是阻塞位，若仍有不可证位保持拒绝）。`generic_call_binding_unproved`/`field_generic_body_unproved`（同类引用的绑定证明）为独立的第二链——同类调用位把泛型投影与重载消歧耦合，属既有签名证明程序的下一前沿。

## 处置方向（串行两片）

- **`recover-nested-generic-class-headers`（先）**：嵌套类的 Signature 头投影（`static class Box<U>` 于折叠文本/分离文本均可投影）——`class_generic_source_unproved` 的 nesting/kind/name 位证明；解锁 U 域链与折叠评估。
- **`recover-same-class-generic-bindings`（后）**：同类 Methodref/Fieldref 的泛型投影绑定证明（重载消歧位）。

原 class 为行为基准。
