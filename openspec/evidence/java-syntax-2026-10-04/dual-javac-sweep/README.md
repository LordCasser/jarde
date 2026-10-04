# 双 javac 系统性扫描（2026-10-04，root）——版本耦合盲区的判定规则与全量清点

**背景**：本会话已连续发现三例同一根因的缺口（[DT-03 `getClass` null-check](../qualified-outer-alloc-getclass-patrol/README.md)、[返回值形写访问器](../value-returning-write-accessor-patrol/README.md)、[CF-17 TWR codegen](../twr-javac8-codegen-patrol/README.md)）：语料由 javac 9+（含 `--release 8` 交叉编译）产出，故真 javac 8 的 codegen 形对既有验收结构性不可见。三例逐个发现效率低，本扫描改为**系统性双腿普查**，一次覆盖 10 类 Java 8 常见构造。

**产出（比缺口清单更有价值的是判定规则）**：`--release 8` **会**回退"API 表面"差异，但**不会**回退"编译器内部 codegen"差异——这精确预测了哪些构造有盲区。见第二节。

扫描器 [sweep.py](sweep.py)（10 探针 [probes](probes/)、22 份双腿渲染 [results](results/)、汇总 [results/sweep-summary.txt](results/sweep-summary.txt)）。**扫描器已用已知正例自检**：单独跑 `P08_twr`（已知版本耦合）正确报 `VERSION-COUPLED`，故其零结果可信。

## 一、扫描结果（10 构造 × 双腿）

| 探针 | 构造 | 真 javac 8 引注 | javac 23 引注 | 判定 |
| --- | --- | --- | --- | --- |
| P01 | 字符串拼接 `"v="+x+":"+y` | 0 | 0 | **OK**（双腿都恢复为 `+` 源码形） |
| P02 | lambda 捕获数组元素复合赋值 `t[0]+=i` | 1 | 1 | BOTH-REFUSE（**版本无关**的能力缺口） |
| P03 | 方法引用 `System.out::println` | 3 | 3 | BOTH-REFUSE（版本无关，属 DT-27 已登记残留） |
| P04 | String switch | 0 | 0 | OK |
| P05 | 枚举常量体 + abstract 方法 | 0 | 0 | OK |
| P06 | 匿名类捕获 final 局部 | 0 | 0 | OK |
| P07 | 内部类读写外部私有字段 + `o.new In()` | **13** | 3 | **BOTH-REFUSE 且程度版本相关**（见第三节） |
| P08 | try-with-resources | **1** | **0** | **VERSION-COUPLED** |
| P09 | 泛型 + bridge（`T extends Comparable<T>`） | 0 | 0 | OK |
| P10 | 两层嵌套匿名接口 | 0 | 0 | OK |

双腿 `java -Xverify:all` 行为输出**全部相同**（`behavior_same=True` ×10），故十个探针都是合法基线、差异只在呈现侧。

**清点结论**：10 构造中**纯版本耦合仅 1 例**（P08 TWR），**版本相关程度差异 1 例**（P07），**版本无关缺口 2 例**（P02/P03），其余 6 例双腿健康。即版本耦合**不是普遍现象**，而是特定构造的 codegen 属性——这正好由第二节的规则解释。

## 二、判定规则：`--release 8` 回退 API 表面差异，不回退编译器内部 codegen 差异

root 实测 `P01` 字符串拼接（javac 9+ 的重大变化是 `invokedynamic` + `StringConcatFactory`）：

| 腿 | `StringBuilder` 引用数 | `invokedynamic` 引用数 |
| --- | --- | --- |
| 真 javac 8 | 14 | 0 |
| javac 23 `--release 8` | **14** | **0** |

即 `--release 8` **把字符串拼接回退成了 `StringBuilder` 形**——因为 `StringConcatFactory` 是 **JDK 9 才有的 API**，不在 Java 8 的 API 表面内，`--release` 会据此回退。故 P01 双腿同形、都恢复，**无盲区**。

对照本会话三例缺口，其 javac 9+ 变化**都不是 API 表面差异，而是编译器内部 codegen 决策**，`--release 8` **不回退**：

| 构造 | javac 9+ 的变化 | 性质 | `--release 8` 是否回退 | 有盲区？ |
| --- | --- | --- | --- | --- |
| 字符串拼接 | `StringBuilder` → `invokedynamic StringConcatFactory` | **API 表面**（JDK 9 新 API） | **是** | 否（P01 实证） |
| 限定外部实例 null-check | `Object.getClass()` → `Objects.requireNonNull` | **内部 codegen**（`Objects.requireNonNull` 自 JDK 7 就存在，非新 API） | **否**（root 实测三组合） | **是**（DT-03） |
| try-with-resources | `aconst_null` 副本 + `ifnull` 守卫 → 简化关闭序列（48→22 指令） | **内部 codegen**（JDK 9 的 TWR 重写） | **否** | **是**（CF-17） |
| 写访问器 | void `(LC;D)V` → 返回值形 `(LC;D)D`（`dup_x1`） | **内部 codegen**（栈重排表达赋值结果） | **否** | **是**（EM-15） |

**规则（可直接用于预测下一个盲区）**：若某构造的 javac 9+ 差异是**调用了 JDK 9 才引入的 API**，则 `--release 8` 会回退、既有 javac-23 语料**能**代表真 javac 8，无盲区；若差异是**编译器内部的指令序列/栈形选择**（不依赖新 API），则 `--release 8` **不回退**，既有语料**不能**代表真 javac 8，**有盲区**。

**推论（须优先排查的构造类）**：凡 javac 9+ 做过"内部 codegen 优化"且**不涉及新 API** 的构造都在风险区。已知候选：TWR（已证）、限定外部实例 null-check（已证）、访问器栈重排（已证）；**尚未排查**的高风险候选——`switch` 的 table/lookup 阈值与 `default` 分支布局、字符串 switch 的 `$SwitchMap` 合成数组形态、`synchronized` 的 monitor 序列、lambda 的 `altMetafactory` 参数与捕获形、自动装箱的 `valueOf` 缓存路径、`assert` 的 `$assertionsDisabled` 合成字段形。这些属后续扫描范围，本巡查不外推（**未实测的构造一律不下断言**）。

> **状态更新（2026-10-04 同日）**：上段"尚未排查"的六项候选**已由 [codegen-differential-sweep](../codegen-differential-sweep/README.md) 全部排查完毕，均无版本盲区**（整类 opcode 序列 SHA 双腿一致）；lambda 亦已补测（opcode 序列 + `BootstrapMethods` + `InnerClasses` + 合成方法命名四维一致，无盲区）。该普查同时暴露了 opcode-序列扫描器对 `invokedynamic` 系构造的盲区（BSM 参数须属性级比对），已登记。故本节推论的候选清单**已结清**；剩余待排查方向见该普查第三节末（compact-string、nestmate、接口私有方法等，均未实测、不外推）。

## 三、P07 的"程度版本相关"（两缺口的叠加，非第四例）

P07（内部类读写外部私有字段 + `o.new In()`）真 javac 8 引注 **13**、javac 23 引注 **3**——两腿都拒，但真 javac 8 拒得更多。root 核实这是**已登记两缺口的叠加**，不是第四例版本耦合：

- javac 23 腿的 3 处引注 = **返回值形写访问器**缺口（EM-15，`access$` 体不恢复 → 空 stub）。
- 真 javac 8 腿多出的 10 处 = **`getClass` null-check** 缺口（DT-03，`o.new In()` 整方法不恢复，连锁引注其后续语句）。

故 P07 是"DT-03 + EM-15 两缺口在同一探针上的叠加呈现"，其证据已分别归档于两个专项巡查，不重复登记。

## 四、扫描中发现的**版本无关**缺口（顺带产出，须独立处置）

双腿引注数相同 → 与 javac 版本无关，是纯能力缺口：

1. **P02 lambda 捕获数组元素复合赋值**（`int[] t={0}; l.forEach(i -> t[0]+=i)`）：`lambda.rs:774` 拒绝，理由 `captured operand 0 is \`Obje

> **P02 形态扩样（root 2026-10-04 追加巡查，多语句 lambda 体前沿）**：构造 5 形探针（单语句块体/两语句块体/条件体/三语句引用捕获/块+return 体）实测——**引用捕获的多语句体全部健康**（`fmt` 三语句、`ret` 块+return 均恢复为 companion-call 形 `(Object p0) -> ML.lambda$fmt$3$jarde(local1, …)`，物理 helper 照常呈现），**多语句前沿无独立缺口**；而三个原生数组捕获变体（单语句/两语句/条件体）全部命中**同一** P02 拒绝（帧 `Object` vs `int[]`）——即已立项的 [recover-lambda-primitive-array-capture](../../changes/recover-lambda-primitive-array-capture/) 的锚家族实际覆盖 `sum`/`sum2`/`cond` 三形态（lambda 体复杂度不影响该拒绝，判别变量纯是捕获类型来源），修复后将一并恢复。负结果（多语句体健康）不立项。ct\` in the frame, \`int[]\` in the site descriptor and \`int[]\` in the implementation: the capture conversion and its creation-time effects are not …proved`。即捕获操作数在 SSA 帧中是 `Object` 而在站点描述符与实现中是 `int[]`，捕获转换未获证。**root 查重：未登记**（`preserve-array-invocation-widening` 是数组实参扩宽，非 lambda 捕获；DT-26 的验收锚是"准确 BSM、直接参数来源"的捕获 lambda，不含数组捕获）。归属 DT-26，属**未登记的响亮拒绝**，待独立取证立项。
2. **P03 方法引用 `System.out::println`**（绑定接收者取自**他类静态字段**）：`getstatic System.out; dup; invokestatic Objects.requireNonNull; pop; invokedynamic` 被拒。**root 查重：已登记**——即本会话 [method-reference-patrol](../method-reference-patrol/README.md) 的形 2「绑定实例引用 `m::inst`」，其拒绝**有原则**：Java 的 `m::inst` 在**创建函数值时**即对 `m` 做 null 检查，改写成 `(p) -> m.inst(p)` 会把 NPE **推迟到调用时**，异常时机不同，故拒绝而非产出"可编译但异常语义不同"的文本。属 DT-27 明文登记的残留、优先级为呈现润色。**本扫描的价值是新增一个数据点**（绑定接收者来自 `getstatic` 他类字段这一子形同样落在该有原则拒绝内），不改变其归属与优先级。

> **顺带登记（与 P03 同族，值得将来一并取证）**：P03 的字节码显示**绑定方法引用的接收者 null-check 也用 `Objects.requireNonNull`**（`dup; invokestatic Objects.requireNonNull; pop; invokedynamic`）。这与 DT-03 片的 `requireNonNull` 判据是**同一 JDK 惯用法族但不同消费位**（DT-03 是限定分配的 null-check，P03 是绑定方法引用接收者的 null-check）。故真 javac 8 下 P03 该处应为 `getClass()` 形——但 P03 双腿引注相同（均 3），说明**拒绝发生在 null-check 之前**（`invokedynamic`/绑定引用本身未获证），故 DT-03 片**不会**顺带改变 P03 的呈现。这一因果关系 root 已实测确认，登记以免将来误判"DT-03 片应修复 P03"。

## 五、处置

- **TWR（P08）**：已登记 CF-17 已证差距（[control-flow.md](../../jadx-feature-inventory-2026-09-27/control-flow.md) 未完成目标段 + [专项巡查](../twr-javac8-codegen-patrol/README.md)），**尚未立 spec**——须先取证 `guard::Shape::NullableResourceFinally`（`guard.rs:363`，构造点 `3853`）与 `region.rs:5252` `nullable_resource_finally_regions` 能否扩展到 JDK 8 的 `aconst_null`+`ifnull` 守卫形，判定是窄扩展还是机制工作（48 vs 22 指令、16 vs 7 异常表项，root 初判偏中大颗粒）。
- **P02 lambda 数组捕获**：新登记的版本无关缺口，归 DT-26，待取证立项。
- **P03**：已登记（DT-27 有原则拒绝），仅新增数据点，不改优先级。
- **横切方法学（本扫描的主要产出）**：第二节的判定规则应作为**将来所有涉及 javac 合成 codegen 的巡查的前置检查**——先判"该构造的 javac 9+ 差异是 API 表面还是内部 codegen"，若是后者则**必须双腿取证**。已固化进 [handoff.md](../../../../handoff.md)（"字节码惯用法普查必须按指令序列匹配"纪律含双 javac 腿要求）。是否对全语料做一次真 javac 8 双腿补强，属独立大颗粒项（会改变全部字节 SHA、fingerprint 与以 SHA 断言的测试），须单独评估。

原 class 为行为基准：十个探针的双腿运行输出逐探针相同，见 [results/sweep-summary.txt](results/sweep-summary.txt) 的 `behavior_same=True`。
