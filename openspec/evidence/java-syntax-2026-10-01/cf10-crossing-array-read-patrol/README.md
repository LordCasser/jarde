# CF-10 巡查：跨保护区域局部的数组读写值（2026-10-01）

[CF-10 账本](../../jadx-feature-inventory-2026-09-27/control-flow.md)登记"`i += 2` 完整类缺口在局部定义/使用跨 quoted region 的呈现，内部失败路径尚未追踪"的定向取证（主线 `a0486f3b`）。固定转录 [fixture](fixture/)（F1/F2，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| F1.stepTwo / stepTwoWithCall：`i += 2` 计数循环，体为算术/调用语句，**无保护区域** | 完整恢复（正确保持计数循环，行为一致 `9`/`3915`）——`i+=2` 本身已不构成差距 |
| F2.stepTwoWithQuote：`i += 2` 循环，体内含 try/catch，`sum` 跨界（写@15 在 try 前、写@24 在 try 内、写@29 在 catch、读@10/16/38） | 整方法拒绝："local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write" |

## 根因（失败路径追踪）

`build.rs::all_reads_reach_presented_writes` 的**写值形状白名单**（`presented_int_store_value`：非共享 int 字面量、同块局部 load、静态调用、加法树）不含**数组元素读**（`iaload` 等）。F2 write@15 `sum = sum + data[i]` 的加法树含 `iaload@13` → 无法判定为可呈现写值 → `DeclarationPlacement::Incomplete` → 拒绝。F1 对照证明同表达式在无跨界门时呈现正常；界门（crosses_exception → 路径证明）设计合理，缺口纯在白名单形状。读侧 phi 链（循环头/汇合点合并 entry 写、try 内写、catch 内 iinc 写）在现有 walk 内闭合。

## 处置方向

`recover-crossing-array-read-values`：写值白名单接受"同块数组元素读"——数组操作数与下标表达式均在既有白名单内（参数/局部/加法树/常量）、单用途、块内求值位置不变（与既有内联纪律一致）。数组元素类型经既有 `array_of_value`/descriptor 通道。预期 F2 完整恢复且行为一致（`14`）；F1 逐字不变；白名单不放宽共享值/跨块/副作用形态（负例钉死）。

原 class 为行为基准；JADX 参照不作为语义正例。附带观察：enumswitch pass 对非静态字段数组读的 `jre_enumswitch_shape` 拒绝为无害噪声（F1 恢复场景同在），不在本片范围。
