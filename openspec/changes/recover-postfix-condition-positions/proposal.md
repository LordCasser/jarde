# 后缀自增条件位恢复（recover-postfix-condition-positions，postfix 快照片 Phase B）

## Why

[postinc-condition 巡查](../../evidence/java-syntax-2026-10-05/postinc-condition-patrol/README.md)（Phase B 冻结面）：**条件位三形**——do-while 扫描 `while(xs[i++] != 0 && …)`、while 复合条件、if 短路条件位——现全部整方法拒（"local crosses a quoted fallback region"，local-crossing 门）。`recover-postfix-old-value-snapshot` A 相已交付快照消费位机制（`prove_local_snapshots`/`snapshot_consumer`/吸收感知/事后记账），Phase B 显式延后：条件位的消费方是**控制流分支**而非值表达式，且 local-crossing 门额外参与。jadx 全解。

## What Changes

- A 相快照证明的消费者泛化到**条件位**：`iinc` 旧值 load 跨 iinc 被循环/分支测试读取（`ifle/ifne/ifnull` 等）时，测试表达式按 `xs[i++] != 0` 源码形呈现；
- local-crossing 门的对应豁免：已证快照的 `iinc`+其旧值 load 属同一证明单元（吸收感知已在 A 相建立——核对 `test_is_pure`/crossing 判据是否需同 A 相的成对豁免）；
- MVP：单变量单条件位；多变量复合条件、嵌套循环条件位、短路链中段形如实记录后按需另片。

## 硬不变量

1. A 相全部锚（局部/下标/三元/数组存 RHS）渲染逐字节不变；
2. `i = i++` 等自赋陷阱与多消费方形保持拒绝；
3. 不得产出"可编译且行为不同"文本（do-while 扫描的求值次数=迭代数，行为对比必测）。

## 验收

- 三形恢复（0 local-crossing/0 快照诊断），剥离编译 exit 0、`-Xverify:all` 输出与原一致（巡查值）；A 相套件零回退；全门禁 + oracle ignored 腿 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：后缀自增旧值作循环/分支条件操作数时按源码形态呈现，方法行为完整。
