# 引用/数值相等实参位布尔呈现 —— 判定点与变体证据（`recover-ref-eq-boolean-argument`）

主线 `8ffe22a8` 之上的实现切片。锚定场景为 [S1](../fixture/S1.class)（BCI 148–156：`if_acmpne 155; iconst_1; goto 156; iconst_0; append(Z)`，行为基线 [orig.out](../fixture/orig.out) 尾段 `falsetrue`）。

## 判定点定位（任务 1.1）

- **拒绝产生点**：`crates/jarde-java/src/build.rs` `Builder::invocation_argument` —— `presented=int`、`required=boolean` 走 `Conversion::Unspellable` 分支，拒绝信息 `…declared \`boolean\` presents \`int\` and this layer has no proven conversion to \`boolean\``（S1 BCI 156，见 [../fixture/S1.jarde.java](../fixture/S1.jarde.java) 引注）。
- **既有布尔证明接入位**：同文件 `build_conditional_value` 的分支值塌缩（`Z` `ireturn` 位：`test.presented == Boolean` + 双臂 `integer_constant` 1/0 → 塌缩为 `test` / `Not{test}`）。实参位在此接入：`boolean_position_values` 把塌缩判定的消费者从"`ireturn` + `Z` 返回"扩到"调用实参 + 该实参位形参 `Z`"（`equality_argument_parameter`：消费者 `Operation::Invoke` 的形参表与栈实参按 `typed_arguments` 同一位移映射对位，`phi` 所在位形参为 `Z` 且分支为 `JumpIfSame`/`JumpIfDifferent`——`if_acmpeq/ne`、`if_icmpeq/ne`）。判据（双臂 0/1 合流 + 布尔测试）复用同一塌缩代码路径，未复制第二份；边界（`<`、零测试、null 测试）不进。

## 变体/负例（任务 1.2；均 javac `--release 8` 编译、`java -Xverify:all` 运行通过）

| 名 | 源形态 | 降低形态 | 实现前 | 实现后 |
| --- | --- | --- | --- | --- |
| S1 | `b.append(t == t.intern())` | `if_acmpne` 双臂 → `append(Z)`（BCI 148–156） | 语句被引 | `local1.append(local2 == local2.intern());` |
| R1 | `b.append(t != s)` | `if_acmpeq` 双臂 → `append(Z)` | 语句被引 | `local1.append(local2 != arg0);` |
| R2 | `mark(t.length() == s.length(), s.isEmpty())` | `if_icmpne` 双臂 → `mark(ZZ)` 第 0 参 | 语句被引 | `return mark(local1.length() == arg0.length(), arg0.isEmpty());` |
| R3 | `boolean r = t == s; b.append(r);` | 双臂 → `istore` → `iload` → `append(Z)` | 已恢复（`boolean local3 = local2 == arg0;`） | 逐字一致（存储位属声明计划，未动） |
| N1 | `take(s.length() < 3)` | `if_icmpge` 双臂 → `take(Z)` | 拒绝（"no proven conversion to \`boolean\`"） | 同前逐字（序分支不在本切片） |
| N3 | `take(s.length() == 0)` | `ifeq`（零测试）双臂 → `take(Z)` | 拒绝 | 同前逐字（零测试非 `if_icmpXX`） |
| N2 | `take(p & q)` | `iand` → `take(Z)`（非比较来源） | 已恢复（`return take(arg0 & arg1);`，位运算自身呈布尔） | 同前逐字 |

逐文件前后输出：[before/](before/)（实现前 class-source）、[after/](after/)（实现后 class-source；S1 的"前"即巡查冻结的 [../fixture/S1.jarde.java](../fixture/S1.jarde.java)）。fixture 与 SHA：[results/fixture-sha256.txt](results/fixture-sha256.txt)。回归检查：`crates/jarde-java/tests/p3_repeq_boolean_arguments.rs`（7 用例）。

## 三方对照（任务 3.2；原 class / 固定 JADX dev Java-input 重编 / Jarde 重编，均 `java -Xverify:all`）

| 场景 | 原class / JADX / Jarde 运行 SHA256（一致） |
| --- | --- |
| S1 | `056ba9ac…b8edbf7`（[S1-original-run.txt](results/S1-original-run.txt) / [S1-jadx-run.txt](results/S1-jadx-run.txt) / [S1-jarde-run.txt](results/S1-jarde-run.txt)） |
| R1 | `21d91041…bea8220b` |
| R2 | `8d3cae7e…06f05e9` |
| R3 | `7cb5205a…c3a3d38` |
| N1/N2/N3 | 原class = JADX 重编（`82c1315e…`、`5d90ef7f…`、`82c1315e…`）；Jarde 保持拒绝/既有恢复，无重编路径 |

JADX CLI：`/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx`（dev）。重编均 `javac --release 8`。
