# 短路布尔局部的分支位读取（recover-short-circuit-local-branch-reads）

## Why

[operator-remainder 巡查](../../evidence/java-syntax-2026-10-05/operator-remainder-patrol/README.md)的 `OP2.condAssignOld`（`boolean b = (x += 1) > 0 && x > 0; return b ? x : -1;`）与 [census 重跑](../../evidence/java-syntax-10-07/diagnosis-census-rerun/README.md)（22 行，收口后最大单点残余）实证：`proves_boolean_local_store` 的消费者白名单只认 `putstatic Z` / `ireturn Z` / `append(Z)`（StringBuilder/StringBuffer），**分支消费位缺失**——已证布尔局部被 `ifeq`/`ifne` 读取（javac 把 `b ? x : -1`、`if (b)`、`while (b)` 都分解为分支测试）时整链拒绝。jadx 有解（`return b ? x : -1` 源码形）。这是 `recover-scv-concat-consumers`（拼接位扩展）的姊妹片：同一白名单的下一消费位。

**判别（完整）**：`return b;`（ireturn-Z，已恢复）/ `flag + text`（append-Z，已恢复）/ `b ? x : -1`（ifeq，**拒**）——唯一变量是消费者种类。同型 `if (b) {...}` 与 `while (b)` 同因。

## What Changes

- `proves_boolean_local_store` 的 `boolean_position` 白名单增加**分支臂**：消费者为 `ifeq`/`ifne` 且读的是该已加载布尔值（加载即消费，分支不重复求值、不改求值序）；
- 三元形（`b ? x : -1`）呈现为已存局部的条件值读（复用既有 `Conditional`/条件值呈现——布尔局部已是 proven `Type::Boolean`，无需新类型证据）；
- 语句 `if (b)` / 循环 `while (b)` 形由既有 If/Loop 条件呈现自然承接（测试钉住）；
- 白名单其余判据（单写、声明区、同词法 region、trivial merge 容忍、预算）逐字不动。

## 硬不变量

1. 既有三个消费位（putstatic-Z/ireturn-Z/append-Z）锚渲染逐字节不变；
2. 跨词法 region 读保持拒绝（gate 的 `access.path != write.path` 注释明说留 local-scope 域）；
3. 不得产出"可编译且行为不同"文本（分支读不改求值序——load 位置不动）；
4. `recover_short_circuit_local_values` 既有套件零回退。

## 验收

- `OP2.condAssignOld` 恢复（0 该诊断），整类剥离编译 exit 0、`-Xverify:all` 输出与原一致（含 `condAssignOld(0)` 值）；新增 if/while 条件位锚同验；
- 门控实验先行（只加分支臂，OP2 翻转/跨 region 负例不翻）；
- 全门禁 + oracle ignored 腿 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：已证短路布尔局部的分支条件读取（三元/语句/循环条件位）按源码形态呈现，方法行为完整。
