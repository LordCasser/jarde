# Design：静态泛型字段初始化的损坏文本修复

## Context（root 已实测，全部在 [巡查](../../evidence/java-syntax-2026-10-05/generic-static-field-init-patrol/README.md)）

- 损坏样例（逐字）：`static Hold f1 = new MN$Holdava.lang.Object) "a");`——`new MN$Hold` 与 `(java.lang.Object)` 的左括号丢失、类型实参丢失、嵌套形出现 `Hold.lang.Object`。
- 触发：静态字段 + `field_generic_body_unproved`（字段泛型 Signature 投影拒绝，`src/class_source.rs` ~5891）+ 初始化表达式含泛型类构造调用。菱形/显式实参同坏；实例字段不受影响（走构造器赋值退化）。
- jadx 有解（`new MN.Hold<>("a")` 完整形）。

## 决策 1：修复形态 = 裸类型正确形（路径 A 优先），实现者可按代码实情选 B

呈现必须满足：**可编译**（`javac --release 8` exit 0）或**响亮引注**（拒绝 + bytecode 标记）——两者之外的任何中间态（现状的损坏文本）都是缺陷。优先路径 A：`static Hold f1 = new Hold((java.lang.Object) "a");`——

- 类型名用池形（`Hold`/或裸 `MN$Hold`，与全仓呈现约定一致）；
- 构造器实参按擦除描述符 `Ljava/lang/Object;` 呈现（既有 `(java.lang.Object)` cast 先例遍布全仓）；
- 与实例字段 f3 的退化路径**同构**（f3 已证明该形合法可编译）。

## 决策 2：落点在 class_source.rs 的初始化回退分支（锚点名引用）

`field_generic_body_unproved` 拒绝后，静态字段的初始化表达式走了与 f3 不同的呈现路径（f3 归构造器赋值；f1 试图内联到字段声明处）且内联路径的文本拼接有 bug（丢 `(`）。**插桩/读码确认内联拼接点**（root 未定位到具体拼接行——实现者 task 1.1 定位并转录），修复 = 拼接点产出的文本必须与路径 A 一致，或该分支直接拒绝引注。

## 决策 3：验收锚与零回退

- 主锚：`MN`（`f1`/`f2` 静态菱形+显式）修复后 `javac --release 8` exit 0、`main` 输出 `a b 5` 与原 class 一致；
- 零回退：`f3`（实例字段退化）逐字节不变；`RG` 探针的宿主类渲染中**非静态泛型字段**不受影响；corpus 双腿扫描——**差异类只能是含静态泛型字段初始化的类**（预期极少数，语料普查 0 亦可，如实记录）；
- 负例：泛型 Signature 投影**拒绝注释保留**（本片不修投影本身，只修文本）。

## 验证标准（可证伪）

1. 主锚双腿（javac23 `--release 8` 编译的 MN + 真 javac 8 编译的 MN）渲染：无损坏文本（`grep -c 'Holdava'` = 0）、`javac --release 8` exit 0、行为一致；
2. 零回退：f3 与 corpus 差异边界如上；
3. 门禁：全量测试（基线 302/2976+EM-15 落地后数字）、fmt、CI-exact clippy、openspec strict、fingerprint 再生。

## Open Questions

1. 内联拼接点的具体位置与 bug 机制（task 1.1 插桩/读码定位）；
2. 静态字段初始化是否有 f3 式构造器赋值通道可复用（路径 B 可行性）——实现者判断，报告说明。
