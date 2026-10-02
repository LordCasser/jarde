## ADDED Requirements

### Requirement: do-while(false) 体内的 return 可恢复且引注失败闭合

系统 SHALL 在 do-while(false) 降低体的语句集（含条件 return 与 break 出边）自洽时，将该区域按 `do { … } while (false);` 呈现。当区域证明失败且引注区包含控制流改变出边（return/throw/break/continue 的目标不在已证呈现内）时，系统 SHALL 整方法拒绝——不得产出剥离引注后仍可编译的部分体。既有 do-while、双跳转与普通 return 形态 SHALL 逐字不变。

#### Scenario: 静默错编消除

- **WHEN** `while (…) { i++; do { if (i%3==0) break; if (i>7) return "early"; } while(false); }` 三方 Java 8 重编运行
- **THEN** do-while 与 return 完整呈现且运行输出 `early` 与原类一致；或证明失败时整方法拒绝——任何情况下不得出现"可编译且行为不同"

#### Scenario: 既有形态不变

- **WHEN** 输入为普通 do-while、双跳转循环或 for/if 内 return
- **THEN** 输出与本变更前逐字一致
