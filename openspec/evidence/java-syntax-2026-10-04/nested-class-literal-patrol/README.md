# 嵌套类的类字面量巡查（2026-10-04）

**实现结果（2026-10-04，`recover-nested-class-literal-values`）**：见 [results-values/README.md](results-values/README.md) —— 判据放宽前后引注对照（A12 6→0、N2 10→0、A11 15→0、A9 17→0、A10 零回归字节一致、LC 负例保持）、三方对照、corpus 465 类双腿扫描 0 差异、新增冻结 fixture 与 SHA、遗留单列（折叠深度域、折叠数组 token-tie 锚域）。

注解/反射域巡查引出（主线 `30e54613`；**巡查用 spn worktree 的 HEAD 版 jarde-cli，零额外构建**）。固定转录 [fixture](fixture/)（A10 类字面量五形对照 / A11 反射注解三形 / A12 嵌套判别 / A9 复合；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 o2/o3/o4.out。

## 判别矩阵（单变量钉死）

| 场景 | 主线 Jarde |
| --- | --- |
| **顶层类字面量**五形：`A10.class` 作实参/receiver/拼接 receiver/链式 `.getName().length()`/存局部再读 | **全部恢复**（`A10`/`A10`/`A10!`/`3`/`A10`） |
| **嵌套类字面量**：`Nested.class.getSimpleName()`、`Nested.class.getName()`（A12 两形） | **整方法拒绝**：BCI 0 `ldc class A12$Nested` — "the instruction at BCI 0 is not part of the provable subset"，级联"dependency chain … not bounded" + "saved declaration could not commit" |
| 反射读注解（`Marked.class.getAnnotation(Tag.class)` / `isAnnotationPresent` / `((Tag) a).value()`） | 三形全拒——**根因即嵌套类字面量**（`A11$Marked`/`A11$Tag`），非注解或反射本身 |
| 注解声明本体、`@Meta(value=…, n=…, tags={…})` 使用、默认值、`@Deprecated`/`@SuppressWarnings` | 恢复（A9$Target/A9$DefTarget 零引注） |

## 根因

`ldc` 的 CONSTANT_Class 名为**嵌套形态**（`Outer$Nested`）时不在可证子集内；顶层名（含当前类名）健康。与 `recover-nested-type-source-spelling`（呈现缝：`$`→源码拼写）相邻但**不同层**——那片处理"已证类型引用的文本拼写"，本片是"类字面量值本身的证明准入"。判据缺口大概率是类字面量准入处的名形/可解析性检查未接受嵌套名（或要求名 == 当前类）。

## 影响面

反射入口 `Foo.Bar.class`（logger、enum 元数据、内部 Builder、注解处理器）为**超高频**形态；当前拒绝导致整个反射方法不发布 → 完整类不可编。A9.main（17 处引注）与 A11 全部三方法同因。

## 处置方向

`recover-nested-class-literal-values`（窄证明层切片）：类字面量准入接受嵌套 CP 名——判据沿用既有类型名可拼性事实（nested-spelling 片的 `source_spellable_member_row`/`nested_reference_spelling` 同源），值按既有类字面量呈现通道（呈现拼写由 nested-spelling 片负责：折叠域内 `Nested.class`、分离域 `Outer$Nested.class` 或点分形按该片既有规则）。A12 两形与 A11 三形恢复；顶层类字面量五形逐字不变；不可拼名（本地类 `1$Local`、匿名 `X$1`）保持拒绝。

原 class 为行为基准。


## 补强 fixture（2026-10-04，实现者可直接复用）

[N2](fixture/N2.java)（`fn2.jar`，运行基线 `Leaf`/`N2$Outer$Mid`/`Mid`）覆盖三种更深形态，主线全部拒绝：

| 形态 | 源码 | 主线 |
| --- | --- | --- |
| 多段嵌套 | `Outer.Mid.Leaf.class.getSimpleName()`（CP 名 `N2$Outer$Mid$Leaf`） | 拒绝（3 引注） |
| 中层嵌套 | `Outer.Mid.class.getName()` | 拒绝（3 引注） |
| 链式 receiver | `Outer.Mid.Leaf.class.getEnclosingClass().getSimpleName()` | 拒绝（4 引注） |

三形与 A12 单段形同因（嵌套 CP 名不在可证子集），确认判据须覆盖**任意段数**而非仅一段；链式 receiver 形另需确认准入后依赖链不再触发 "not bounded" 级联（design 已列该风险）。

## 判据的 javac 实证（2026-10-04，root；负例 fixture 已就位）

design 决策 1 的推论（"按 `$` 分段后每段均为合法标识符即准入，本地/匿名类被同一判据自然排除"）经真实 javac 输出确认——非推理：

| javac 生成的二进制名 | `$` 分段 | 每段皆合法标识符 | 判据结果 |
| --- | --- | --- | --- |
| `LC$Inner`（具名嵌套类） | `LC` / `Inner` | 是 | **准入** |
| `A12$Nested` | `A12` / `Nested` | 是 | **准入** |
| `N2$Outer$Mid$Leaf`（多段） | `N2` / `Outer` / `Mid` / `Leaf` | 是 | **准入** |
| `LC$1Local`（**本地类**） | `LC` / `1Local` | 否（数字开头） | **拒绝** |
| `LC$1`（**匿名类**） | `LC` / `1` | 否（数字开头） | **拒绝** |

依据：`names.rs::is_java_identifier`（125 行）要求首字符为 `_`/`$`/ASCII 字母——数字开头即否。故无需专门规则排除本地/匿名形。

[fixture/LC.java](fixture/LC.java) 与四个 class（`LC`、`LC$Inner`、`LC$1Local`、`LC$1`）已冻结，可直接作为 tasks 1.2 要求的本地类/匿名类**负例**输入（`LC$1Local.class` 的 CP 含 `ldc class LC$1Local`；匿名形经 `getClass()` 不产生 ldc，故负例主体是 `LC$1Local`）。

放宽范围核实：`source_internal_name` 在全仓仅两处调用（`decode.rs:322` 数组元素名、`337` 对象名），均在 `class_literal_type` 内——**只影响类字面量准入，不波及其它路径**，确认是单点接线而非跨层改动。
