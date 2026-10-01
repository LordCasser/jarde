# `recover-scv-concat-consumers` 证据（EM-19 巡查缺口收口，2026-10-01）

本目录是 [巡查 README](../README.md) 中"存+拼接 ✗"一行的实现后证据。基线（实现前）即巡查记录本身：B5.s1/s2、B4.v1–v3、B2.compound 整方法退化，B3 与 s3 完整恢复。冻结巡查 fixture 未改动（SHA 见 [../results/fixture-sha256.txt](../results/fixture-sha256.txt)，本轮复核一致）。

## 1. append descriptor 取证（[append-descriptor-javap.txt](append-descriptor-javap.txt)）

javap 冻结 B5.class（SHA 复核 `fccfb351…`）与变体 fixture：javac 8 把 `boolean` 局部的拼接实参一律 lowering 为**原语重载** `StringBuilder.append:(Z)Ljava/lang/StringBuilder;`（B5.s1 消费位 @26；`"" + hasA` 头位探测同样落在 `append:(Z)`，常量 `""` 走 `append(String)`，布尔局部无装箱 lowering）。`append:(Ljava/lang/Object;)` 按验证器语义不可能**直接**读布尔局部的值（中间必有 `Boolean.valueOf` 装箱），故本变更只匹配原语 append 位，不加装箱匹配臂——与 design 决策 1"descriptor 按取证精确写"一致。

## 2. 消费方枚举落点（实现位置）

- `crates/jarde-java/src/region.rs` `short_circuit_value`：消费方锚点枚举/两路分支的 consumer block 终态判定——消费块内含锚点又以下一结构的测试分支结束时，从消费块自身起识别**延续**并组合进本 Region（`tail`），不再整块拒绝；值证明的 consumer 出边守卫同步放宽。
- `crates/jarde-java/src/build.rs` `proves_boolean_local_store`（由原 SCV Local 臂提取）：局部布尔种子的**读位置集合**纳入 `append:(Z)`（StringBuilder/StringBuffer，加载值为其唯一操作数、读全为栈位）；同一纪律经 `proves_conditional_store_boolean` 覆盖折叠证明的**单测**条件存储（B5.s2 的 `hasB` 形状）。
- 呈现零新增规则：声明走既有 `Declare`，拼接走既有 CONCAT 通道（`local1 + ":"`），单测折叠在 boolean 声明处拼写为条件本身。

## 3. 变体前后（[fixture-sha256.txt](fixture-sha256.txt)、各 `*.jarde.java`、`*-run.txt`）

| 形状 | 实现前 | 实现后 |
| --- | --- | --- |
| B5.s1 / `plain`（append(Z) 尾位） | 整方法退化 | `boolean local1 = (arg0 & 1) != 0 && (arg0 & 2) != 0; return "" + local1 + ":";` |
| `boxedHead`（`"" + hasA` 头位） | 整方法退化 | 同上（lowering 仍为 `append:(Z)`，javap 钉死） |
| `doubleChain`（双短路双拼接，第二链 2 测试） | "arms of the branch in block 6 do not meet at one join" | 两个 boolean 声明 + 一条拼接，完整恢复 |
| B5.s2 / `singleTail`（第二条件单测试在第一消费块内） | 同上退化 | `boolean local2 = (arg0 & 4) != 0;`（折叠种子）+ 拼接 |
| B4.v1–v3、B2.compound（复合位前缀/两链） | 整方法退化 | 完整恢复（B2.compound 见 `B2.jarde.java`） |
| `reRead`（拼接前局部经 bastore 二次读） | 整方法退化 | **保持退化**（负例语义不变，[ScvConcatReReadControls.jarde.java](ScvConcatReReadControls.jarde.java)） |

判别链锚定：B3（直接 return）与 B5.s3（存+return）恢复文本与实现前**逐字节一致**（`diff` 断言，测试锚定于既有 `p3_mixed_short_circuit_local` 与本次 B3 复核）；B1 位运算主行文本逐字节不变。

三方对照（`java -Xverify:all`）：**原类 / 固定 JADX 重编 / Jarde 重编逐路径一致**——B2/B4/B5 的 `*.jadx-run.txt`、`*.orig-run.txt`、`*.recovered-run.txt` 三方 SHA 完全相同（见 [fixture-sha256.txt](fixture-sha256.txt)），JADX 输出冻结于 `*.jadx.java`；恢复源以 `javac --release 8 -g:none` 重编通过。冻结变体 `scv-original-run.txt`/`scv-recovered-run.txt` 相同（`true:\ntrue:\nfalse:true\nfalse:true`）。

## 4. 测试

`tests/p3_scv_concat_consumers.rs`：四正例文本断言（Structured、无 fallback、无 `? 1 : 0`/`% 2 != 0`）、负例退化与 source map 覆盖、整类重编 + `-Xverify:all` 双方运行对照、预算耗尽与取消原子性。fixture 冻结于 `tests/fixtures/p3-conditional-values/scv-concat-consumers/`（javac 23 `--release 8 -g:none`，README 内含命令与 SHA）。
