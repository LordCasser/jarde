# 枚举 switch 常量标签巡查（2026-10-04）——归入既有未完成 change，不新立 spec

枚举 switch 域巡查（主线 `196185fd`；巡查复用 spn worktree 的 HEAD 版 jarde-cli——已核实二进制忠实反映 HEAD）。固定转录 [fixture](fixture/)（E1 嵌套枚举双 switch / E2+Color 跨类枚举 / `E2$1` 合成表类；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 orig.out（`g3rl`/`b2xh`）、o2.out（`rg?`/`15:101`）。

## 结果矩阵

| 场景 | 主线 Jarde | 性质 |
| --- | --- | --- |
| E1（同文件嵌套枚举，直接 `ordinal()` 派发、无 `$SwitchMap`） | 恢复，`switch (arg0.ordinal()) { case 0: … }` 整数标签 | 可编译、行为正确；常量名未拼写 |
| E2（跨类枚举，javac 合成 `E2$1` 持 `$SwitchMap$Color`） | 恢复，`switch (E2$1.$SwitchMap$Color[arg0.ordinal()]) { case 1: … }`；**`E2$1` 作为独立类呈现（源码无此类）** | 可编译、行为正确；非源码形 |
| 同方法双 switch（`twice`）、同方法双枚举 | 全部恢复（`project-multiple-enum-switch-sites` 已 8/8 完成，双站点 blanket refusal 已收敛） | 已闭合 |
| 诊断 | `jre_enumswitch`："the constants those entries stand for are the enum class's own declaration, **which this run does not hold**, so no `case T.CONST:` label is written" | 证明未成立 |

## 归属判断（关键）

**缺口已由 [project-proved-enum-switch-labels](../../../changes/project-proved-enum-switch-labels/) 精确覆盖，状态 1/9（仅 1.1 基线已勾）**——其 Why 段与本巡查结论逐字对应：

> "现有 `enumswitch@1` 如实恢复该整数选择，却不能把 `case 1/2` 写为 `case RED/BLUE`"；方案为"按选定运行环境有界地读取真实表定义、初始化体、枚举常量及 `values()` 数组来源，证明每个已用整数键与一个常量的对应关系"。

因此**本巡查不新立 spec**（避免与既有 change 重复立项——本会话已在 bridge 域因先查既有 change 而避免同类重复）。本片为既有 change 的**证据补强**：

- E2 是该 change 设计目标的**跨类依赖读取**典型场景（表在合成类 `E2$1`、枚举 `Color` 独立顶层类、switch 在 `E2`——三方分属不同类），可作为其 1.3 永久 fixture 的候选输入（其现有 `enum-switch-labels/` fixture 为单 helper 形）。
- E1 补充一个**无 `$SwitchMap` 的直接 `ordinal()` 派发**对照形：该形不经 helper 表，其标签证明只需枚举常量声明顺序（`case 0` → 第 0 个常量 `RED`），依赖面显著小于 E2——建议该 change 实施时按此**分层**（直接 ordinal 形先行，跨类 helper 形随后），降低单次交付风险。
- 诊断文本本身已指明阻塞在"this run does not hold the enum class's own declaration"，即依赖读取未接入，与其 tasks 2.2（"使用现有环境/reader 按需解析表定义与 enum 类型"）一致。

## 处置

- **不新立 change**；把本巡查证据登记为该 change 的输入（E1/E2 fixture + 分层建议），待队列排到该 change 时由实现者按 tasks 1.2–3.3 推进。
- 该 change 为**大颗粒**（跨类 IR 证明：枚举 `<clinit>`、`(String,int)` 构造器、`values()` 数组工厂、helper 表 `<clinit>` 异常表），需最高思维强度与充足配额窗口。
- 保真度缺口（非可编译性缺口）——E1/E2 当前输出可编译且行为正确，优先级低于本轮已立的 bridge name-clash（硬编译错误）与 nested-class-literal（整方法不发布）两片。

原 class 为行为基准（`g3rl`/`b2xh`、`rg?`/`15:101`）。
