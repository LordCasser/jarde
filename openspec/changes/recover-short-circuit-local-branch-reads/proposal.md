# 短路布尔局部的分支位读取（recover-short-circuit-local-branch-reads）

## Why

[operator-remainder 巡查](../../evidence/java-syntax-2026-10-05/operator-remainder-patrol/README.md)的 `OP2.condAssignOld`（`boolean b = (x += 1) > 0 && x > 0; return b ? x : -1;`）与 [census 重跑](../../evidence/java-syntax-2026-10-07/diagnosis-census-rerun/README.md)（22 行，收口后最大单点残余）实证：`proves_boolean_local_store` 的消费者白名单只认 `putstatic Z` / `ireturn Z` / `append(Z)`（StringBuilder/StringBuffer），**分支消费位缺失**——已证布尔局部被 `ifeq`/`ifne` 读取（javac 把 `b ? x : -1`、`if (b)`、`while (b)` 都分解为分支测试）时整链拒绝。jadx 有解（`return b ? x : -1` 源码形）。这是 `recover-scv-concat-consumers`（拼接位扩展）的姊妹片：同一白名单的下一消费位。

**判别（完整）**：`return b;`（ireturn-Z，已恢复）/ `flag + text`（append-Z，已恢复）/ `b ? x : -1`（ifeq，**拒**）——唯一变量是消费者种类。同型 `if (b) {...}` 同因。

## What Changes

- `proves_boolean_local_store` 的 `boolean_position` 白名单增加**分支臂**：消费者为 `ifeq`/`ifne`（`0x99`/`0x9a`，且解码语义为 `JumpIfZero`/`JumpIfNotZero`）且读的是该已加载布尔值（加载即消费，分支不重复求值、不改求值序）；
- 三元形（`b ? x : -1`）呈现为已存局部的条件值读（复用既有 `Conditional`/条件值呈现——布尔局部已是 proven `Type::Boolean`，无需新类型证据）；实测：三元两臂（`x` / `-1` 物化）由既有条件值通道完整承接，**不需要本片之外的任何东西**；
- 语句 `if (b)` 形由既有 If 条件呈现自然承接（测试钉住）；`ifne` 形（源码 `if (!b)`）按实际分支语义拼写为两臂互换的 `if (!b)`；
- 白名单其余判据（单写、声明区、同词法 region、trivial merge 容忍、预算）逐字不动。

## 测量更正（实施期实测，2026-10-07，与立案时的两处推断不同）

1. **循环条件位 `while (b)` 不在本臂可达范围内**：立案时推断它"由既有 Loop 条件呈现自然承接"。实测（`results/01-refusal-chain.out`）：`while (b)` 的读取落在循环**头块**——它是自身的 canonical block，`collect_paths` 把循环头归给循环区域，故读取路径 `[1]` ≠ 声明路径 `[0]`，gate 的**跨词法 region 判据先于消费者白名单**拒绝该局部（`crossed=true`）。呈现层确实承接了循环条件（今日已渲染 `while (b != 0)`，局部类型 `int`），拒绝发生在更早的一道判据上，而那道判据正是本片的硬不变量 2（"跨词法 region 读保持拒绝"；代码注释原文已把跨 region 读留给后续 scope 变更：*"A read in another lexical region needs an independently proved elevated assignment and is conservatively left to a later scope change."*）。故循环条件位在本片**保持拒绝并登记为边界**，spec 以独立 Scenario 钉住；交付它需要"跨词法 region 的提升赋值"另立切片。
2. **链中段读不被区域判据先拒**：立案时按"可能先被短路区域判据拒绝"预警。实测：`(x > 0) && b && (x < 100)` 的 `b` 与三元同路到达消费者白名单，分支臂准入，第二条链的区域合成把它拼成嵌套形 `x > 0 && (b && x < 100)`（求值序与源码一致）。作为正例锚冻结。

## 硬不变量

1. 既有三个消费位（putstatic-Z/ireturn-Z/append-Z）锚渲染逐字节不变；
2. 跨词法 region 读保持拒绝（gate 的 `access.path != write.path` 注释明说留 local-scope 域）——实测形态：循环头读（路径分歧）与 catch 内写/ try 外读（更早的 region ownership 层）；
3. 不得产出"可编译且行为不同"文本（分支读不改求值序——load 位置不动）；
4. `recover_short_circuit_local_values` 既有套件零回退。

## 验收

- `OP2.condAssignOld` 恢复（0 该诊断），整类剥离编译 exit 0、`-Xverify:all` 输出与原一致（含 `condAssignOld(0)`；`main` 属拼接 saved-producer 族已登记残余，replay 以冻结源码自身的 `main` 体替换 refused 标记后编译，替换在测试与 README 中明写）；`BranchReads` 三个条件位锚同验（整类自身可编译可运行）；`BranchReadNegatives` 两条边界逐字拒绝且剥离文本不可编译；两个 opcode 补丁控制钉住分支臂宽度；
- 门控实验先行（只加分支臂，OP2 翻转/循环与跨 region 负例不翻/既有三消费位逐字节）；
- 全门禁 + oracle ignored 腿 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：已证短路布尔局部的分支条件读取（三元/语句/链中段条件位）按源码形态呈现，方法行为完整；循环条件位与跨词法 region 形保持拒绝。
