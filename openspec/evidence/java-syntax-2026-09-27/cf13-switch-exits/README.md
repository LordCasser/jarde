# CF-13 switch 出口审计（2026-09-27）

Jarde 基线：`c0a8c147ead5c1b067070441764b7ad17253e0cf`，分支 `codex/cf13-switch-exit-audit`。固定 JADX checkout `/Users/lordcasser/workspace/testzone/jadx`，HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`，工作区干净。

## 固定测试与实现 SHA / 断言强度

| 文件 | SHA-256 | 实际断言 |
|---|---|---|
| `TestSwitchBreak.java` | `47b2d3c471513977b32f30d69a89934d71b1c48a0c9cf0dc95732c3fe44bd2f7` | `TestCls.check()` 期望 `test(9)` 返回固定字符串；主测试检查 switch/case/break 文本，并对 switch 导致的带标签 loop exit 留 TODO，预期用 `return s + "+";` 简化。 |
| `TestSwitchContinue.java` | `1df9311b5618977046e61f60ff1f1f43dac3918f7c48e1c12824757f2ff9ed80` | 仅检查 switch/case、两处 break、`a -= 2;` 和一处 continue 文本；没有 `check()` 行为断言，也没有整类编译断言。 |
| `TestSwitchReturnFromCase.java` | `c557be310ea8a49165a695c1acdb009351b748f07a11176742f46ce15dced7bf` | 仅检查 case 数量、break 数量及赋值文本；没有 `check()`。 |
| `TestSwitchWithThrow.java` | `018eecf874b087669d0ab18c31e4f771334b52fe2d34818e77c7b614ede8383a` | `check()` 验证正常返回与异常类型/消息；主测试只检查一个 throw 文本。 |
| `TestSwitchInLoop4.java` | `9e3bce89a9d79911b850efeb0c95a6b80b85020b3ee8ce01e7bf09e7f1f651ab` | `check()` 对有/无分隔符分别断言 true/false；每个 profile 的文本断言仅要求一个 switch 与一个 break。 |
| `SwitchBreakVisitor.java` | `7f49a6a38c2b11846501063fe234a098a35bb1b54b9dddc6d2efa04f3f50dbf5` | switch 后处理在 LoopRegionVisitor 之后运行；在分支均有 break/exit 时抽取 common break，并移除紧跟 return/throw/continue 的 unreachable break。 |
| `SwitchRegionMaker.java` | `741133377c6d9ec0f60c40d234f01e3dbd79f2bd80e4a2ed15ca105f0a408bd1` | 由 case 支配前沿计算 switch out；只有恰好两个出口且其中一个是 loop end 时，尝试在对应 case 插 continue 后将 loop end 从 switch out 集合移除。 |
| `RegionGen.java` | `8b80ef618487e38cb09a2bbd81b70de0dd096996a321a0c9dcf55e1dc011d4ee` | switch region 输出路径。 |

以上 SHA 已逐文件对固定 checkout 核对。

## 窄 Java 8 对照与差距

`input/SwitchLoopExits.java` 是可终止的 counted loop。case 0 的 `break` 只退出 switch，随后执行 `score += 3`；case 1 的 `continue` 跳过该语句并前进到外层 loop update；default 的 break 也只退出 switch。它把 switch break 和外层 loop continue 分开观察，没有掺入 return、throw 或嵌套 switch。

原始 class 通过 `javac --release 8 -g -Xlint:-options`，`java -Xverify:all` 输出 `38`。JADX 的完整源码在 `javac --release 8` 失败：它将 case 1 输出为 `continue;` 后又留下不可达的 `break;`（生成源码第 16 行）。Jarde 也未生成完整方法：`class-source` 将整个 `walk(I)I` 标记为 explanation-only，诊断为 `canonical block at BCI 9 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted`；其完整 class-source 因该方法无返回语句而未通过 Java 8 编译。故此三方没有可比的反编译运行结果；原始行为 `38` 是已验证的唯一运行值，不把输出文本近似当作等价。

JVM 指令结构可用 `javap -c -classpath original cf13.SwitchLoopExits` 重现：BCI 9 是 switch selector load，BCI 12 是 `lookupswitch`；case 0 在 BCI 40 跳到 BCI 55 的 switch 后共享语句，case 1 在 BCI 46 跳到 BCI 58 的 loop update，default 从 BCI 52 落入 BCI 55；BCI 61 回到 loop 条件 BCI 4。也就是说同一 switch 的 case 分别通向 switch-local join 与外层 loop update，Jarde 在 selector 所属 canonical block 的 region 所有权检查处拒绝完整方法。这个具体组合是可重放的 recoverability gap，不是已观察到的 Jarde 错误运行结果。

JADX 固定测试的文字断言不足以发现它留下的 unreachable `break;`。Jarde 现有 `p3_switch_loop_exits` 3/3 通过：已有正例核验 loop-break 的 BCI 归属，已有负例明确要求没有唯一 in-loop join 的 nested-switch/continue 保持 quoted。当前 fixture 没有 nested switch，故它给出了更窄的 switch-local break + outer continue + shared tail 触发条件。一个独立处理方向是单独扩展 switch/loop 出口所有权证明：为 switch-local join 与 loop-update edge 分别确认唯一 owner，再恢复 switch；不要把 continue case 改写成 switch break，也不要将其后的共享语句并入 continue 路径。这里仅记录差距，不改生产或提出 OpenSpec。

输入 SHA-256：`7f2e679f5c7ae6452fdb624901cb04c371c17e01082b38be8c53a3b8f3919071`。JADX、Jarde 源码和原始 class 保存在本目录；两个失败的编译器输出日志也一并保留。

复现的核心命令：

```sh
javac --release 8 -g -Xlint:-options -d openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/original openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/input/SwitchLoopExits.java openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/input/SwitchLoopExitsRunner.java
jadx -d openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/jadx openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/original/cf13/SwitchLoopExits.class
cargo run -q -p jarde-cli -- class-source --input openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/original/cf13/SwitchLoopExits.class --class cf13.SwitchLoopExits --policy single-class --release 8 --format text > openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/jarde/SwitchLoopExits.java
javac --release 8 -g -Xlint:-options -d openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/jadx-classes openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/jadx/sources/cf13/SwitchLoopExits.java openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/input/SwitchLoopExitsRunner.java
javac --release 8 -g -Xlint:-options -d openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/jarde openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/jarde/SwitchLoopExits.java openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/input/SwitchLoopExitsRunner.java
java -Xverify:all -cp openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/original cf13.SwitchLoopExitsRunner
```

Existing Jarde verification: `CARGO_TARGET_DIR=/tmp/jarde-cf13-target CARGO_INCREMENTAL=0 cargo test --test p3_switch_loop_exits` passed 3/3. The temporary Cargo target is removed after this audit.
