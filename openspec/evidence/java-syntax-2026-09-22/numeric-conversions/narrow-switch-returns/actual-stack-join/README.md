# Actual stack-join `ireturn` audit

这是对父目录初步 local-join 证据的有界补充。`ActualStackJoin.java` 先用
`javac --release 8 -g:none` 编译成合法的普通局部变量版本；`patch_stack_join.py` 再对
三个 `Code` 做精确字节修改：删除两臂的 `istore_1` 和 join 的 `iload_1`，保留两臂
常量在操作数栈上，改写 `lookupswitch`/`goto` 偏移，使两臂共用唯一的 `ireturn`，并将
class major 52 改为 49、移除 `StackMapTable`。这一步只改变已核对的 fixture，不是生产
解析器。

## 重放

在仓库根目录执行：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/numeric-conversions/narrow-switch-returns/actual-stack-join/run_audit.py
```

脚本保存源码 class、精确 patch JSON、完整 `javap`、原始/JADX/jarde 的 class-source
文本和编译/运行日志。descriptor patcher 会分别产生三组 `(I)B`、`(I)C`、`(I)S`
变体；每组只修改对应的 `runByte`、`runChar` 或 `runShort` descriptor，且检查所有
`Code` attribute 的字节完全不变。固定 CLI `/tmp/jarde-cli-deferred-final-ecab` 的
SHA 前后都记录为
`ecab8244d1709765fa0f2b0effcebe49b9f066eea911843d11d34abec5837330`。

## 字节码事实

源码 class 为 414 bytes，SHA-256 为
`3e2e2970faa2a1432867c403cdd34387cd7f1beac9b9d7d0fa04a03cc9ca5b4b`。stack-join patch
产生 372 bytes、major 49、SHA-256
`adaf8142090cc1c25f41e597350ca6ff31baf17081aeb8f0f89accc512f51e98`。三个方法的源码和
补丁 `Code` 长度都保持不变；每个方法都移除了 1 个嵌套 `StackMapTable` attribute：

| 方法 | switch BCI | 源码 arm value BCI | 补丁 default value BCI | 唯一 join `ireturn` BCI | Code 长度 |
| --- | ---: | ---: | ---: | ---: | ---: |
| `runByte(int)` | 1 | case 20 / default 27 | 26 | 32 | 33 |
| `runChar(int)` | 1 | case 20 / default 26 | 25 | 30 | 31 |
| `runShort(int)` | 1 | case 20 / default 26 | 25 | 30 | 31 |

补丁后的实际形状可由 `i-stack-join/stack-javap.txt` 复核：每个 case/default 先产生
常量，随后 `goto` 到同一 `ireturn`；没有 arm store、join load 或额外 return。major 49
且无 StackMapTable 的 class 通过 `java -Xverify:all`，12 行 runner 输出与
`i-stack-join.expected.txt` 相等。

## 三方结果

先验证原始 `(I)I` class，再验证 stack-join `(I)I` class。两者的 runner、JVM、完整
JADX 重编译运行和完整 jarde 重编译运行都为 exit 0；原始局部形状的 jarde 仍呈现局部
赋值，stack-join 形状则实际呈现每个 arm 的 `return`，quote 数均为 0。这个顺序证明
恢复依赖真实的栈 join，而不是由父目录的局部 `iload` 形状推断出来。

随后对该同一 stack-join Code 分别、独立地改一个返回 descriptor。每个单变体新增一个
descriptor UTF8，因此三组 class 各为 379 bytes，SHA 分别为 B
`ad9b88a41ec56ed7a5ff553388dcf3867f8d00bc5a6a27404da4976b7b684149`、C
`c5c2ab16469319970488a56cb881cb14873a23c55e59e77b4dd582f1dd3b7abe`、S
`a5f141e48dc0ee777e02d9cd4c87c25a27ca3b9b40f86909f8f05e2153196dd3`。三组均通过
`java -Xverify:all`，12 行输出分别符合
JVMS 的 B/C/S 窄化；JADX 完整输出也均编译运行成功。当前 jarde 能识别真实 join 的
`@bytecode`（每组 1 个 quote），但没有把栈入口值发布成返回表达式，生成文本在三个
方法上都因“缺少返回语句”而编译失败。这是本轮观察到的恢复边界，未用错误 Java 文本
冒充成功。

完整机器可读结果在 [`summary.json`](summary.json)，三组 descriptor 的 method/offset
记录在各自的 `descriptor-patch.json`，stack rewrite 的 BCI 和 hash 在
[`stack-patch.json`](stack-patch.json)。
