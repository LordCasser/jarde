# Test11 双循环 finally 恢复验收

固定类 `TestTryCatchFinally11$TestCls.class` 的 SHA-256 为 `3a67a7f63596c5ca7b7dbdb97a2027224d69f2a1d7b21e4f598d897510bc5f92`。`test(List)` 的 33 个物理 BCI 是 `0,1,4,5,10,11,12,17,20,21,26,27,28,29,32,35,38,40,41,46,48,50,55,58,60,65,67,68,70,73,76,78,79`，仅有 `[0,4)→38 any` 与 `[38,40)→38 any` 两行。后一行只保护 BCI 38 的原异常绑定；两条清理循环都在这两行之外。固定探针以 `javac --release 8` 编译，验证 `test(List)` 与固定类有相同 BCI 和异常表，只有 BCI 1/29/70 的调用 opcode 因私有方法从 `invokevirtual` 变为 `invokespecial`。

用当前 `replay.sh` 与 fresh Jarde CLI 完整运行后，固定 TestCls 的原 class、固定 JADX 完整源码、Jarde 完整源码均用 Java 8 源级别重编，`java -Xverify:all` 输出均为 `two:102`、`empty:100`。Jarde 的 `test` 只含一个 `try/finally`，finally 内一条 `Iterator` 循环和一次 `call2(item)`。Rust 定向测试检查 8 个 canonical block 各有一个区域所有者，全部 33 个 BCI 都在 source map 中。生成源码使用等价的显式 `while` 循环；并未声称还原原源码的 `for-each` 拼写。

扩展探针使清理调用也能抛错。`FinallyLoopCases` 让同一 `test(List)` 布局中的 `iterator`、`hasNext`、`next`、`call2` 分别在正常与正文异常入口抛错；原/JADX/Jarde 三份完整源码重编后的 10 条 `-Xverify:all` 输出逐字一致：

| 清理路径 | 正文正常 | 正文先抛 `body` |
|---|---|---|
| 全部成功 | `normal:false:102:ok` | `normal:true:102:IllegalStateException:body` |
| `iterator` 抛错 | `iterator:false:100:IllegalStateException:iterator` | `iterator:true:100:IllegalStateException:iterator` |
| `hasNext` 抛错 | `hasNext:false:100:IllegalStateException:hasNext` | `hasNext:true:100:IllegalStateException:hasNext` |
| `next` 抛错 | `next:false:100:IllegalStateException:next` | `next:true:100:IllegalStateException:next` |
| `call2` 抛错 | `call2:false:101:IllegalStateException:call2` | `call2:true:101:IllegalStateException:call2` |

`make-neighbors.py` 从 Java 8 探针构造并冻结七个近邻，具体 SHA-256 见 [neighbors/SHA256SUMS](neighbors/SHA256SUMS)。`replay.sh` 先核 SHA，再逐个装载对应类，以 `java -Xverify:all` 执行正常路径，并要求 Jarde 不生成 `finally`：

| 近邻 | 改动 | 当前边界 |
|---|---|---|
| `handler-second-entry` | 正常清理的 BCI 32 直入异常循环 BCI 58；同步修改局部槽与 StackMap，形成两个外部入口 | `jre_region_irreducible` |
| `different-list` | 可实现 `List` 的接收者替代 handler 副本的入口参数 | 无 finally 证书 |
| `different-target` | handler 逐项调用改为同签名 `call3` | 无 finally 证书 |
| `different-loop-test` | handler 的 `ifeq` 改为 `ifne` | 无 finally 证书 |
| `cleanup-self-protected` | 第二异常行扩到清理循环 | 无 finally 证书 |
| `throwable-rewritten` | 原 Throwable 读取改为 `aconst_null; nop` | 无 finally 证书 |
| `extra-normal-exit` | 正常循环 BCI 32 提前跳到返回 | 无 finally 证书 |

局部循环证明只在方法入口不可达的 normal-flow 组件上寻找唯一根；`dominates` 仍以方法入口为根。异常边直入组件内部、无根闭环、循环第二入口以及预算/取消都有定向拒绝测试。FINALLY 证书借鉴固定 JADX `MarkFinallyVisitor.processTryBlock`/`findCommonInsns` 从异常重抛沿出口寻找重复清理的次序；Jarde 额外检查两行范围、同一入口值、调用符号、SSA 生产消费、所有 canonical 边与全指令归属，一次提交 Region/Builder 输出，不沿用 JADX 的通用 `DONT_GENERATE` 标记。

实施门禁：`cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`openspec validate recover-handler-loop-finally --strict` 与 `git diff --check` 均通过。Test11 的 `replay.sh` 通过上述三方完整回放；Test13 五行 finally 的 acceptance 与有效负例、Test16 六路径、Test17 八路径、旧 nested finally 的 `replay.sh` 也通过。普通循环、catch-join、不可约边界及两/三/四/五行 finally 的 Rust 回归均包含在整套 `jarde-java` tests 中。唯一未完成的是 OpenSpec 4.3 的 root 独立验收。
