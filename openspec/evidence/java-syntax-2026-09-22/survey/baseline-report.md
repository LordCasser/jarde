# SyntaxProbe：Java 8 恢复巡查

日期：2026-09-22（Asia/Shanghai）

## 范围与输入

fixture 源码：[src/SyntaxProbe.java](/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-22/survey/SyntaxProbe.java)，使用 `javac --release 8 -g:none` 编译；没有执行外部目标。原始 class 只有一个文件：[classes/SyntaxProbe.class](/tmp/jarde-syntax-probe-20260922/classes/SyntaxProbe.class)（1204 bytes）。字节码基准见 [javap.txt](/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-22/survey/javap.txt)。

jarde 主结果使用已存在且较新的 `/Users/lordcasser/workspace/projects/jarde/target/debug/jarde-cli`；release 结果只作为历史对照，保存在 `jarde-release-*`。两个 CLI 的 `class-source --help` 已保存。调用是 standalone class + `--policy single-class --release 8`，分别保存 [jarde-text.txt](/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-22/survey/jarde-text.txt)、[jarde.json](/tmp/jarde-syntax-probe-20260922/jarde.json) 与 stderr。jadx 为 `/opt/homebrew/bin/jadx` 1.5.6，源码在 [jadx/sources/defpackage/SyntaxProbe.java](/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-22/survey/jadx.java.txt)。完整可重放命令见 [replay.sh](/tmp/jarde-syntax-probe-20260922/replay.sh)。

## 总体结果

class-source 成功（text/JSON exit 0），读取 15 个方法。debug jarde JSON 报告为 2 个 `structured/java`（构造器和普通数组 helper）和 13 个 `fallback/mixed`、`syntax_status=not_java`；全部 `verification=not_performed`，没有可用于判断“错值”的已验证 jarde 表达式。jadx 对 14 个静态方法都重建了完整 Java；它省略了构造器，因为隐式 `Object` 构造器足以表达同一默认行为。jarde 在 `loopConditionCall` 保留了 `local1 = 0`，但循环和返回均未恢复，并明确写了 fallback 标记，因此不能把这个局部片段当作完整源码。

## 逐方法对照

下面的“源码”是 fixture 中的核心语句；jadx 列出反编译后的实际语句；jarde 列出实际恢复状态及关键 BCI/诊断。参数名变化（源码名 → jadx 名 → jarde `argN`）是命名差异，不是值差异。

| 方法 | 源码核心 | jadx | jarde 具体结果 / fallback 诊断 |
|---|---|---|---|
| `<init>()V` | `public SyntaxProbe() { super(); }` | 整个构造器被省略（使用隐式默认构造器） | `structured/java`；输出 `super(); return;`。`jre_constructor_prologue` 指出 BCI 1 调用 `java/lang/Object.<init>`；这是唯一完整 Java 体。 |
| `unaryInt(I)I` | `return -(-value);` | `return -(-i);` | `fallback/mixed`，无表达式；BCI 1、2（两次 `ineg`）不在 provable subset，BCI 3 的返回值来自 BCI 2 的 `Other`。 |
| `unaryLong(J)J` | `return -(-value) * (100L / -value);` | `return (-(-j)) * (100 / (-j));` | `fallback/mixed`，无表达式；两次 `lneg`（BCI 1、2）和右操作数 `lneg`（BCI 7）未恢复，BCI 10 `lreturn` 来自 BCI 2 的 `Other`。 |
| `unaryFloat(F)F` | `return -(-value) * (100.0f / -value);` | `return (-(-f)) * (100.0f / (-f));` | `fallback/mixed`，无表达式；`fneg`/右操作数链的 BCI 1、2、3、6 不在 subset，BCI 9 返回值来自 BCI 2 的 `Other`。 |
| `unaryDouble(D)D` | `return -(-value) * (100.0d / -value);` | `return (-(-d)) * (100.0d / (-d));` | `fallback/mixed`，无表达式；BCI 1、2、3、7 不在 subset，BCI 10 返回值来自 BCI 2 的 `Other`。 |
| `negativeZeroFloat()F` | `return -0.0f;` | `return -0.0f;` | `fallback/mixed`，无表达式；常量加载 BCI 0 不在 subset，BCI 2 返回值来自 BCI 0 的 `Other`。jarde 没有产生可核验的负零值。 |
| `negativeZeroDouble()D` | `return -0.0d;` | `return -0.0d;` | `fallback/mixed`，无表达式；BCI 0 常量加载不在 subset，BCI 3 返回值来自 BCI 0 的 `Other`。没有 jarde 负零值可核验。 |
| `checkAndCast(Ljava/lang/Object;)Ljava/lang/String;` | `if (value instanceof String) return (String) value; return null;` | 同一 `instanceof`、`(String)` cast 和 `null` 分支（参数名为 `obj`） | `fallback/mixed`，无表达式；`instanceof` BCI 1 不在 subset，分支/`checkcast`/return 的 BCI 0、7、12、4 被标为由该 `Other` 产生。没有错误 cast 值被输出，整个方法被省略。 |
| `compareLong(JJ)Z` | `return left < right || left == right;` | `return j < j2 || j == j2;` | `fallback/mixed`，无表达式；debug `jre_region_loop` 报 BCI 17 可重入但不属于已证明 loop，另有 `lcmp` BCI 2 不在 subset、BCI 0/6/16/17/3 汇合/entry 诊断。 |
| `compareFloating(DD)Z` | `return left <= right || left != right;` | `return d <= d2 || d != d2;` | `fallback/mixed`，无表达式；同样触发 `jre_region_loop`（BCI 17），`dcmpg`/`dcmpl` BCI 2 不在 subset，分支汇合/entry 状态未恢复。 |
| `conditionValue([II)I` | `return values[index];` | `return iArr[i];` | `structured/java`；debug 实际输出 `return arg0[arg1];`。同时保留 `jre_enumswitch_shape` 说明：读数组不是本轮验证的静态 dispatch table；`jre_enumswitch` 报 1 个 read 中 1 个 refused，但没有阻止普通 `iaload` 语句恢复。release 历史结果曾将它 fallback，见 `jarde-release-*`。 |
| `loopConditionCall([I)I` | `int index=0; while (conditionValue(values,index)<3) index++; return index;` | 完整恢复同一循环（参数名 `iArr`、局部 `i`） | `fallback/mixed`，只保留 `int local1 = 0;`；`jre_region_unmet_precondition` 明确 BCI 2 的测试块含 BCI 4 调用，未满足 loop@1 规则；`jre_region_uncovered_blocks` 列出 `[2, 11, 17]`。循环和 `return` 缺失，文本带 `not_java` 标记，因此是可见的“不完整恢复”，不是错误的完整值。 |
| `bitMix(II)I` | `return (left & right) | ((left ^ right) << 1) | (left >>> 2);` | 同一 `&`、`^`、`<<`、`>>>`、`|` 表达式（参数名 `i`,`i2`） | `fallback/mixed`，无表达式；位操作相关 BCI 2、5、7、8、11、12 不在 subset，BCI 13 返回值来自 BCI 12 的 `Other`。 |
| `arrayInitializer()[I` | `return new int[] {1, 2, 3, -4};` | `return new int[]{1, 2, 3, -4};` | `fallback/mixed`，无表达式；debug 将 `newarray` BCI 1 标为“产生了 body 不读取的值”，`dup`/`iastore` 链 BCI 3、7、11、15 属于未验证形状，BCI 6、10、14、19、20 的值来自未恢复 `Duplicate`。没有数组内容错值输出。 |
| `throwParameter(Ljava/lang/String;)V` | `throw new IllegalArgumentException(message);` | 同一 `new IllegalArgumentException(str)` 且传入参数 | `fallback/mixed`，无语句；`jre_new_shape` 指出 BCI 0 的构造实例只被 BCI 8 的 `athrow` 使用，因而没有 body 位置可写；`jre_new_sites` 为 1 candidate、1 refused，BCI 0/3/5/8 均保留解释。 |

## 结论与边界

这组输入同时显示了“jadx 能重建语法”与“jarde 在未证明形状处保守 fallback”的边界。已确认的 jarde 行为是拒绝或标注未恢复，而不是输出一个可执行的错误表达式；本轮没有 jarde 错值可报告。最明显的结构性差异集中在一元负号/负零、`instanceof`/`checkcast`、长浮点比较、普通数组读被 enumswitch 规则观察、带调用的循环条件、位运算、数组初始化和 `new` + `throw`。

所有原始 stdout/stderr、exit 文件、JSON/text、字节码与反编译结果均留在 `/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-22/survey/`；工作区生产 Rust、spec、tasks 未修改。
