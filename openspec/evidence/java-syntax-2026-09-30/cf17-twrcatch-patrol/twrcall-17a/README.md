# TWR 正文调用语句切片（`recover-twrcall-statement-bodies`，CF-17a）证据 — 2026-09-30

[巡查 README](../README.md) 根因 1 的实现侧证据：`guard.rs` TWR 体证明的语句子集在既有白名单之外新增**一个**接受分支——`pop`（opcode 0x57）后唯一读值是同块紧邻前一条非 void `invoke` 的写值（SSA same，且该写值以本 `pop` 为唯一读者），即 Java 语句表达式 `r.toString();` 的字节码形态（P3 2c.31 的丢弃形状）；呈现层零改动（该通道已存在）。实现落点 `crates/jarde-java/src/guard.rs`：`Facts::statement_carried`（白名单原样提出）、`Facts::discarded_call_pop`（新判据）、`Facts::statement_free_with_discarded_calls`（仅 TWR 体检查点换用，monitor/finally 四个 `statement_free` 调用点一字未动）；回归 `tests/p3_twr_discarded_call.rs`。

## 判据与负例的映射（tasks 1.2）

判据四要件 → 每个负例钉死一个（`original/N17a.class`，JDK 23 Class-File API 手工下级化，[N17aGen.java](N17aGen.java) 可复现；TWR 行对/关闭/抑制/重抛与 javac 23 `--release 8` 布局逐字节同构）：

| # | 形状 | 落空的要件 | 实现前 | 实现后 |
| --- | --- | --- | --- | --- |
| N2 `popWrongValue` | `invoke; dup; pop; pop` | SSA 身份（第一个 pop 读 dup 写值；第二个 pop 读调用写值但前邻是 pop） | `jre_guard_body`@BCI 8 回退 | 逐字节不变（`N17a-recovered.base.txt` vs [N17a-recovered.txt](results/N17a-recovered.txt)，`diff` 为空） |
| N3 `popAfterCast` | `invoke; checkcast; pop` | 紧邻（checkcast 隔在调用与 pop 之间） | 同上 | 逐字节不变 |
| N4 `popSecondReader` | `invoke; dup; astore_3; pop` | 唯一读者（dup 出的副本被存储——verifier 有效字节码里"结果另读者"只能以 dup 形态出现，拒绝由紧邻要件给出；唯一读者要件为纵深防御，单独触发需要非直线字节码） | 同上 | 逐字节不变 |
| N5 `pop2Discard` | `invokestatic; pop2`（javac 对 `System.currentTimeMillis();` 的原生形态） | pop opcode（双槽丢弃不是本子集的 `pop` 语句） | 同上 | 逐字节不变 |

对照通道 N1：**调用结果被局部接收（无 pop）** —— 主线即经赋值通道恢复（`String s = r.toString();` → `java.lang.String local1 = local0.toString();`，quality=structured，twr@1）。设计文本"赋值接收保持拒绝"与主线 c50e7dbb 事实不符（基线 [V17a-recovered.base.txt](results/V17a-recovered.base.txt) `receivedLocal` 已恢复）；本片处理为：新分支不参与（无 pop）、前后行为逐字节一致、"不呈现部分语句"与"不扩大接收面"满足。字面拒绝将回退主线既有恢复，违反零回退门禁。四要件中"唯一读者"在 verifier 有效直线字节码里由紧邻+身份蕴含（值在栈上只此一份），独立负例不存在，如实记录。

## 基线重放（tasks 1.1）

fixture SHA 与 [results/fixture-sha256.txt](results/fixture-sha256.txt) 一致；巡查冻结 T2 = `d702ff36…` 复核一致。可重放命令（工作树根）：

```
target/debug/jarde-cli class-source \
  --input openspec/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/fixture/T2.class \
  --class T2 --policy single-class --format text
```

主线基线（本目录 [results/T2-recovered.base.txt](results/T2-recovered.base.txt)）：`popBody`/`popBodyVoidTouch` 整方法回退（`jre_guard_body` + `local 0 crosses a quoted fallback region`），`voidBody` 恢复——与巡查矩阵一致。巡查目录缺失的基线 JSON 已补齐：[results/T2-popBody.base.json](../results/T2-popBody.base.json)（`quality: fallback`）、[results/T2-voidBody.base.json](../results/T2-voidBody.base.json)（`quality: structured`），用 `jarde-cli recover … --format json` 于主线二进制生成，可逐字节重放。T1/T3 的占位 JSON 属 CF-17b 域，未动。

## 正例族 V17a（tasks 2.1/2.2）

[V17a.java](V17a.java) → 冻结 [original/V17a.class](original/V17a.class)（+嵌套 `G17`/`G17Impl`），javac 23.0.1 `--release 8 -g:none`。实现前 7 处 pop 体全部整方法回退（[results/V17a-recovered.base.txt](results/V17a-recovered.base.txt)），实现后 **0 处拒绝**（[results/V17a-recovered.txt](results/V17a-recovered.txt)）：

| 变体 | 形状 | 实现后呈现 |
| --- | --- | --- |
| pureCalls | 纯调用语句体（两条） | `local0.toString(); local0.hashCode();` |
| mixedVoidAndCall | void+调用语句混排 | `touch(local0); local0.toString();` |
| callBeforeReturnInside | try 内 return 前混排 | `touch(local0); local0.hashCode(); …`（saved-return 通道接 return） |
| callOnlyReturnInside | 调用语句 + try 内 return | `local0.toString(); … return local1;` |
| staticCall | 静态调用 | `give();` |
| virtualCall | 虚调用 | `local0.toString();` |
| interfaceCall | 接口调用（invokeinterface） | `pick().get();` |
| receivedLocal（对照） | 结果被局部接收 | 赋值通道，逐字不变 |

固定类命中（tasks 2.1）：`T2.popBody` → `try (T2 local0 = new T2()) { local0.toString(); }`，`T2.popBodyVoidTouch` → `{ local0.hashCode(); touch(local0); }`（语句序即字节码序）；`voidBody`/`voidBodyReturnInside` 与基线逐字节一致（[results/T2-recovered.txt](results/T2-recovered.txt) 与 base 的 `diff` 仅含两个目标方法）。

## 三方对照（tasks 3.2，`results/three-way/`）

原冻结 class / 固定 JADX dev（`jadx-cli/build/install/jadx/bin/jadx`）/ Jarde `class-source` 三腿各自 `javac --release 8` 重编后 `java -Xverify:all`。两条腿的 jarde/JADX 源码随目录（`*.jarde.java`/`*.jadx.java`、`V17aRunner.java`）；`run-sha256.txt` 逐路径 SHA：

| 类·路径 | 原 class | Jarde | JADX | 备注 |
| --- | --- | --- | --- | --- |
| T2（normal，唯一可观察路径） | `6d30f106…` | 同 | 同 | main 逐方法打印（`done/in/done/done`） |
| V17a·normal | `8ebc6333…` | 同 | 同 | 十行全同 |
| V17a·boom（注入异常路径：体内 `give`/`touch`/`pick` 抛出 + `close` 抛出，含 addSuppressed 链） | `a46e9821…` | 同 | `0db341cf…`（JADX 自身偏差） | JADX 腿在 close-throw-on-normal-path 的四方法上把 close 异常自抑制（`caught:close|sup:close`）——JADX 参照列自身 TWR 重构缺陷，原类为行为基准（巡查 README 惯例），jarde 列与原类逐路径一致 |

编译腿的披露性机械补丁（不属于本片文本）：主线对 TWR saved-return store 的既有呈现 `Object local1 = "X"; return local1;`（`String` 返回方法不可编译，`T2-recovered.base.txt` 基线同此形态）在三腿 jarde 源码副本中改写为 `return "X";`；测试 `the_recovered_classes_compile_and_run_the_original_paths` 内同一补丁同注释。N17a 负例类本体 verifier 有效可运行（`done`×4，测试钉死），被拒的是恢复而非运行。

## 测试与门禁

- `tests/p3_twr_discarded_call.rs`：T2 popBody 命中与语句序、void 对照逐字、赋值通道对照、V17a 七变体、N17a 四负例（quality=fallback + `jre_guard_body`@BCI 8 + 整体回退文本）、三腿编译运行（normal+boom 逐路径）、预算 output_bytes−1 停止与取消令牌零发布。
- `guard.rs` 白名单原样提出为 `statement_carried`，`statement_free` 语义与四个其余调用点不变。
