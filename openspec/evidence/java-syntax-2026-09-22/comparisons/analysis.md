# long/float/double 比较：已测缺口与 NaN 反例

2026-09-22，用 javac 23.0.1 `--release 8 -g:none` 编译同目录 CompareProbe.java，class SHA-256 为 `be2f6f261c618e69f5ba06388a9da29903cf26fd809a764c739134bcdf7621a6`。21 个分支方法覆盖三种数值类型的六种关系与 `!(a < b)`，另有 int 正面对照和 boolean 返回旁证。当前 debug jarde 在 21 个数值分支和 boolean 返回中均保留引用；int 对照恢复成功。生成的完整 class 文本经 javac 得到 22 个缺少返回的错误，未把这些明确标记的引用当作可编译恢复。

jadx 1.5.6 能生成全部方法的可编译正文，但存在执行差异。把它添加的 `package defpackage;` 去掉以匹配默认包 driver（无方法语义改写），与原 class 分别编译运行 CompareRunner。共 **1,309** 组输入，其中 **34** 组不一致，全部为 float_not_lt/double_not_lt 至少一侧 NaN：原程序返回 7，jadx 返回 9。详细结果在 original.txt、jadx-result.txt、mismatches.txt。

```java
// 源码；fcmpg/dcmpg 后接 iflt，NaN 比较结果为 1，落入返回 7。
if (!(a < b)) return 7;
return 9;

// jadx；NaN 上 >= 为 false，错误返回 9。
return a >= b ? 7 : 9;
```

## 架构判断

`decode.rs` 未接纳 0x94–0x98。现有 condition 从 CompareOp 分支决定 taken/fall-through 极性，然后将单输入与零比较；所以缺口在“比较结果生产者 + 后继零分支”的组合，不是 region 不认识 if。

后续可增加一个忠实的比较结果事实，保存 long/float/double 与浮点 NaN 偏置；在现有 condition 中仅对有证明的比较结果消费者选择 Binary/Not。无需新增 AST、region、通用谓词框架、三元表达式或 Java helper 调用。不可为了拼写使用 `a-b`（long 溢出）或 `Float.compare`/`Double.compare`（带符号零与 NaN 规则不同）。无唯一直接消费证明、其它计算消费或跨位置旧值不成立时，继续引用。

先对整数比较结果确定实际分支极性，再按下表转为 Java 条件。每个操作数仅出现一次，不能拆成 `isNaN(a) || a ... b` 而重复有副作用的操作数。

| 对比较结果的测试 | lcmp | fcmpg/dcmpg | fcmpl/dcmpl |
| --- | --- | --- | --- |
| == 0 | a == b | a == b | a == b |
| != 0 | a != b | a != b | a != b |
| < 0 | a < b | a < b | !(a >= b) |
| <= 0 | a <= b | a <= b | !(a > b) |
| > 0 | a > b | !(a <= b) | a > b |
| >= 0 | a >= b | !(a < b) | a >= b |

这是从 [JVMS cmp 指令](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.dcmp_op) 与 [JLS 数值关系](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.20.1) 推导的候选映射，尚未实现。额外 MappingCheck.java 用显式 -1/0/1 比较定义核对表格的全部 float/double 条目，覆盖两个偏置、六个谓词、每类 81 组输入，共 1,944 项通过；这只验证候选映射，不是 jarde 实现验收。实现任务还须覆盖两个方向的分支、两侧 NaN、正负零、无穷、long 边界、左右调用顺序/异常和来源。boolean 返回样例还涉及 0/1 汇合；不得把该问题与比较条件捆绑。

现有大 change 的 2c.7 曾规定 NaN 为真的极性继续保留比较结果；本次分析证明可以复用 Not 表达，现已由 `recover-numeric-comparison-conditions` 独立提案承接并同步调整规划；该 change 的 planning complete 仅代表文档完成，生产实现和验收尚未完成。

## 重放

```sh
mkdir -p /tmp/jarde-comparison-replay
javac --release 8 -g:none -d /tmp/jarde-comparison-replay CompareProbe.java CompareRunner.java
java -cp /tmp/jarde-comparison-replay CompareRunner
```

CLI 使用同 class 的 single-class/release 8 text 输出；jadx 命令为 `jadx --no-res -d <out> <CompareProbe.class>`。重新编译 jadx 时仅移除自动添加的默认包名。jarde 的 javac 失败记录也随证据保存，不提供伪造的 jarde 执行结果。
