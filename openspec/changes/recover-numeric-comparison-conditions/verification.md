# 数值比较条件：冻结、实现与验证记录

2026-09-23。本记录保留 fixture、修前 CLI 基线，并补充数值比较实现后的定向验证。实现没有
更新 census/fingerprint；3.2 的全局验收、fmt、clippy 和 OpenSpec strict 仍由主代理统一记录。

## 冻结输入

`tests/fixtures/p3-numeric-comparison/` 保留一份冻结 class 和一个源码 driver：

```text
NumericComparisons.class：2319 bytes
class-file version：52.0
Code 属性：35（默认构造器 + 34 个方法）
SHA-256：7ab3d3cc033ce1bed61b60a91a90e1ecf968495081b0578aff22bed716f44aa3
```

前 21 个方法复用 comparisons 审计的三类数值、六关系加反向关系：long 的 `lcmp`，float
和 double 的 `fcmpg/fcmpl`、`dcmpg/dcmpl`。`int_lt` 是 int 对照，`double_boolean` 是
布尔汇合负向形状。`callOrder` 覆盖左右 long 调用顺序，`callThrowLeft` 与
`callThrowRight` 覆盖异常侧和调用次数，`sameBlock` 是同块紧邻唯一零分支消费者的正例，
`booleanMerge` 是结果汇合负例。

fixture 使用以下命令生成：

```sh
javac --release 8 -g:none -d tests/fixtures/p3-numeric-comparison/v8 \
  tests/fixtures/p3-numeric-comparison/NumericComparisons.java
```

原 class driver 使用以下命令执行：

```sh
javac --release 8 -g:none -d /tmp/jarde-numeric-comparison/original \
  tests/fixtures/p3-numeric-comparison/NumericComparisons.java \
  tests/fixtures/p3-numeric-comparison/CompareRunner.java
java -Xverify:all -cp /tmp/jarde-numeric-comparison/original CompareRunner
```

原 class 输出保存在 `/tmp/jarde-numeric-comparison/original.txt`，覆盖 1309 组直接
long/float/double 输入，包括 long 极值、两侧 NaN、正负零、无穷和有限值；同时覆盖调用
顺序、每侧一次求值和左右异常。末尾效果为：

```text
callOrder=7:calls=12
callThrowLeft=IllegalStateException:left:calls=1
callThrowRight=IllegalStateException:right:calls=12
sameBlock=7
booleanMerge=true
```

`javap -classpath /tmp/jarde-numeric-comparison/original -c -p NumericComparisons` 保存在
`/tmp/jarde-numeric-comparison/javap.txt`。其中直接消费者是比较 opcode 后紧邻的零分支。

## 修前三方记录

当前 debug CLI 以只读方式执行：

```sh
target/debug/jarde-cli class-source \
  --input tests/fixtures/p3-numeric-comparison/v8/NumericComparisons.class \
  --class NumericComparisons --policy single-class --release 8 --format text \
  >/tmp/jarde-numeric-comparison/jarde-before.java.txt \
  2>/tmp/jarde-numeric-comparison/jarde-before.report.txt
javac --release 8 -g:none -d /tmp/jarde-numeric-comparison/jarde-classes \
  /tmp/jarde-numeric-comparison/jarde-source/NumericComparisons.java \
  tests/fixtures/p3-numeric-comparison/CompareRunner.java
```

CLI 返回 0；但 21 个直接数值方法都在 BCI 2 保留缺口，没有恢复条件，完整生成文本由 javac
以缺少返回语句拒绝。精确编译日志是 `/tmp/jarde-numeric-comparison/jarde-javac.log`，
没有手工修改正文来宣称通过。三个调用方法保留了左右 producer 调用并引用比较；
`booleanMerge` 和 `sameBlock` 也保留了比较缺口。

JADX 1.5.6 对同一个 class 执行反编译，只删除自动添加的 `package defpackage;` 后编译，
并用 `java -Xverify:all` 执行成功。但它与原 class 有 34 行差异，全部是
`float_not_lt`/`double_not_lt` 的 NaN 输入；差异保存在
`/tmp/jarde-numeric-comparison/jadx-mismatches.diff`。这只是观察结果，不能当作实现预期。

## Rust 测试与实现后验证

`tests/p3_numeric_comparison.rs` 已单文件 `rustfmt`，并覆盖五种比较 opcode、六种零分支
谓词、两种浮点 NaN 偏置、来源映射和拒绝边界。实现后的定向命令和日志如下：

```text
cargo check -p jarde-java --locked                         PASS
cargo test --test p3_numeric_comparison --locked           6 passed, 1 ignored
cargo test --test p3_numeric_comparison --locked -- --ignored 1 passed
cargo test -p jarde-java --test p3_patterns --locked      46 passed
cargo build -p jarde-cli --locked                         PASS
```

日志分别保存在 `/tmp/jarde-numeric-check2.log`、`/tmp/jarde-numeric-final-old-local.log`、
`/tmp/jarde-numeric-final-old-local-ignored.log`、`/tmp/jarde-numeric-p3-patterns.log` 和
`/tmp/jarde-numeric-cli-build.log`。ignored JDK 对照比较原类和真实恢复正文的 1312 行输出：
1309 个 long/float/double 输入组合，加上调用顺序、左异常、右异常三个效果场景；覆盖 NaN、
正负零、无穷、有限值、long 极值、调用次数和异常。

负面边界包含 `dup/pop`、local 中转和 old-local 覆写三种内存 Code probe。old-local probe
在保留原 `lload_0` 栈值后写入 local slot 0，再执行 `lcmp/ifne`，恢复保持 Mixed/Fallback，
并保留比较、分支和返回来源；它没有被放宽为直接条件。booleanMerge 同样保持最终汇合返回
的 Mixed/Fallback，并保留 BCI 2、3、11 的来源。

## 直接消费拒绝边界证据

为验证设计中的“唯一、同块、紧邻零分支”边界，未改写永久 fixture。证据脚本
[`patch_long_eq.py`](../../evidence/java-syntax-2026-09-22/comparisons/refusal-boundaries/patch_long_eq.py)
只在冻结 `long_eq` 的唯一完整 Code 字节序列中插入指令，并调整该 Code 的长度、局部槽数量和
唯一 StackMapTable frame：

| 探针 | `lcmp` 后的字节码 | 零分支 BCI | JVM 校验结果 |
| --- | --- | --- | --- |
| `dup-pop` | `dup; pop; ifne` | BCI 5 | `java -Xverify:all` 通过 |
| `local` | `istore 4; iload 4; ifne` | BCI 7 | `java -Xverify:all` 通过 |

两份派生 class、`javap -v`、校验 stdout 和编译 stderr 位于
`../../evidence/java-syntax-2026-09-22/comparisons/refusal-boundaries/`。校验 driver 对两份
class 都调用 `NumericComparisons.long_eq(1L, 1L)`，输出均为 `7`；原始 fixture 的 SHA-256
仍为 `7ab3d3cc033ce1bed61b60a91a90e1ecf968495081b0578aff22bed716f44aa3`。两种探针都保留
`lcmp`、零分支和返回生产者；它们没有被拿来宣称数值条件恢复成功。

`tests/p3_numeric_comparison.rs` 的两个测试通过同一完整 Code pattern 在内存中构造这两份
探针，断言比较 BCI 2 和零分支 BCI 的 source map 仍有来源，并断言正文不直接发布
`if (arg0 == arg2)`。实施代理及root均已运行这些断言，具体修后结果见上文及下文。

OnlyNarrow 的相邻重载审计属于 overloads 边界证据，不计入本任务 class census；结果和日志见
[`overloads/boundaries/README.md`](../../evidence/java-syntax-2026-09-22/overloads/boundaries/README.md)
及其 `cases-v1/OnlyNarrow/` 副本。

## 主代理独立验收

root复核比较结果的真实唯一SSA读取、同块紧邻准入、共享延期判断、既有分支极性后的NaN映射，以及最终branch BCI传递。没有新增比较结果AST或全图索引。2026-09-23独立运行：

- `cargo test -p jarde-java --locked`：98单元、32恢复、46模式，共176项通过。
- 数值比较、eval-context、new-value、reference-cast、special-dispatch、special-refusal、invocation-arguments及D1来源选择：43项通过；未运行的其它ignored用例不计入。
- 数值比较ignored JDK：1项通过，实际比较1309数值组合及3个效果场景，共1312行。该driver把21个直接比较及3个效果方法的真实恢复正文装入带source-only辅助方法的类；它不是整个混合NumericComparisons夹具的class-source重编译，boolean汇合和裸throw helper仍属该夹具中的范围外部分。
- `comparisons/independent-flow/after/`：另一个自写完整类的CLI输出原样javac成功，从10处引用降为0；242项原始/恢复执行完全一致，覆盖NaN、嵌套long条件、double while及左右调用失败。同class的JADX仍有11个NaN错值。
- `comparisons/lower-stack/`：合法额外底层栈值不阻碍比较组合，原样恢复的单方法重编译2/2一致。`comparisons/old-local/`：合法覆写局部槽变体原class输出7/9/7，恢复明确保留旧读取、比较及分支引用，没有错误使用新arg0。
- `comparisons/rejected-operand/`：一次float调用与尚不支持的fconst_0比较，原class/JADX6行一致；恢复引用包含调用1、常量4、比较5、分支6，延期调用没有从失败链丢失。
- 全仓fmt通过。OpenSpec strict为36通过0失败。严格clippy停在既存`region.rs:1736`的`type_complexity`，没有添加allow，也不声称其后的targets全部检查完成。

原始日志归档在`comparisons/root-regressions/`。数值fixture已经纳入上一轮88类/480Code与205条指纹冻结；本轮throw/final字段/instanceof新增语料由root待冻结后集中更新，task3.2的全局语料收尾仍待完成。专项生产语义验收已通过，与尚在实施的其它语法点区分记录。

## 后续统一语料冻结（2026-09-23）

root在throw实施验收窗口完成新一轮语料冻结：91 class、509 Code、75 handler、231 branch/switch target、8 subroutine。reader重扫通过，fingerprint从205增至219文件，旧条目无修改/删除；五项指纹检查通过。numeric六项与throw相邻回归共同通过（合计61项）；全Java包178项、OpenSpec strict37项通过。strict clippy仍仅观察到已记录的region.rs:1736 type_complexity。命令与完整日志在`../../evidence/java-syntax-2026-09-22/throws/root-regressions/`，本项3.2的统一语料检查已完成。
