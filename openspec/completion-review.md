# 实现完成复核（2026-09-21）

**最新裁决：按约定的四项外部判据，T1–T4 本轮闭环确认完成；三项归档保留。** 本次重新审阅 `f4d1044`、`2895bc4`、`5c35cbf` 的实现并复跑关键门禁，没有重开已通过的原反例。同时新复现 T5：已接受的 `append(int)` 消费 `char` 表达式时仍丢失数值转换，原类与产物可编译但值不同。T5 单独登记后续修正，不能把本轮关闭扩大为 concat 转换契约已无缺口。并行导出与性能专项仍各 0/22。

## 第四轮：本轮关闭判据及独立复跑

复核 HEAD 为 `ad0ffa62c41b6eaf7b88d8039a352eec3e2efd20`；最后生产实现为 `5c35cbf`。本次运行时 `src/`、`crates/`、Cargo 文件与该 HEAD 无差异；已有文档改动保留。本次只更新复核/规划，没有修改 Rust、测试实现或归档。Atlas 返回已有索引的旧行号，下面的定位及结论以当前源码、提交 diff 和实际执行为准。

| 判据 | 本次结果 | 裁决 |
| --- | --- | --- |
| T2 整数比较：两侧常量、相等/大小比较及 int/boolean 对照 | `p3_boolean_contexts` 13 passed，P3 编译执行对照通过；先判 `Test` 再转换，混合 boolean/int pair 已可靠拒绝 | 原判据关闭，`f4d1044`；混合 pair 的后续处置在 `2895bc4` |
| T3 局部 boolean：分支交换、复制链、提升及冲突写入 | `p3_hoisted_boolean` 9 passed；类型决定在声明计划、按 `LocalVariable` 共读；未知/冲突写入不继续发布矛盾赋值 | 原判据关闭，`2895bc4` |
| T4 concat：相邻数值与单段加法、boolean/null/Object、求值/异常顺序 | `p3_concat_conversion` 10 passed，P3 编译执行对照通过；另以自定义 `Object.toString()` 副作用/返回 null 复核，转换发生在后续调用前，原/恢复 5 行 trace 一致 | 原判据关闭，`5c35cbf`；不覆盖下面新发现的 T5 |
| T1 深链：正常完成与停止清理、debug/release 子进程 | debug 进程测试 1 passed；重新 build release 后显式进程测试 1 passed；库侧深链完成与发射中途 `output_bytes` 停止包含在上述 10 项中 | 原判据关闭，`5c35cbf`；CLI 文档交付限制单独保留 |

本次复跑命令（仓库根目录）：

```sh
cargo test --locked --test p3_boolean_contexts --test p3_hoisted_boolean --test p3_concat_conversion
cargo test -p jarde-cli --test task_cli --locked a_deep_concatenation_chain_answers_in_a_subprocess -- --exact
cargo test --test p3_execution_comparison --locked -- --ignored --nocapture
cargo build --release -p jarde-cli --locked
cargo test -p jarde-cli --test task_cli --locked -- --ignored --exact the_deep_chain_answers_in_the_optimized_build
```

结果分别为 **32、1、2、构建成功、1**，全部无失败。本机 JDK 为 OpenJDK 23.0.1，编译对照使用 `--release 8`。全 workspace 的 **1310/0/7** 与 clippy 等完整门禁引用各归档及对应 CI，不把它们写成本轮再次执行的数字；变异证据已核对归档，本轮没有在共享工作树重做变异。

三项归档：[整数比较](changes/archive/2026-09-20-decide-comparison-contexts/verification.md)、[局部类型](changes/archive/2026-09-21-unify-local-type-decisions/verification.md)、[concat 与深链](changes/archive/2026-09-21-re-express-string-concatenation/verification.md)。主 `java8-recovery`、`recovery-validation` 已含相应 delta；历史复核及既有任务勾选不改写。

本次只读查询 GitHub 的实际 head/check-runs：

| 实现 / 归档 head | 任务 | CI |
| --- | --- | --- |
| `f4d1044` / `78c15e0` | 17/17 | [35519791166](https://github.com/LordCasser/jarde/actions/runs/35519791166)，4/4 success |
| `2895bc4` / `2ca09ec` | 21/21 | [35521435501](https://github.com/LordCasser/jarde/actions/runs/35521435501)，4/4 success |
| `5c35cbf` / `c2aa793` | 22/22 | [35524042352](https://github.com/LordCasser/jarde/actions/runs/35524042352)，4/4 success，已不在运行中 |

run 对应包含实现的归档 head，不能写成实现提交自身各有独立 run；这不影响该固定树的门禁证据。

文档收尾校验：OpenSpec active + main **20/20**、archived **26/26**；本轮 14 份文档的 89 个本地相对链接无断链，所改文档 `git diff --check` 通过。下述 T5 脚本已从本 Markdown 提取后实跑，得到表中六行差异。

## T5 / P1：append 的 int 参数转换仍可能丢失（新发现，未关闭）

合法 Java 8 输入，参数是 `char`，实际调用的是既有接受集中的 **`append(int)`**：

```java
public static String value(char c) {
    return new StringBuilder().append((int) c).append("!").toString();
}
public static String prefixed(char c) {
    return new StringBuilder().append("x").append((int) c).toString();
}
```

`value(C)Ljava/lang/String;` 的 BCI 7 为 `iload_0`，BCI 8 为 `invokevirtual StringBuilder.append:(I)Ljava/lang/StringBuilder;`；`prefixed` 对应 BCI 12/13。`char → int` 不需要单独的 JVM 转换指令；Java 表达式重建时仍必须保存这次参数转换。当前产物分别是 `return "" + arg0 + "!";` 和 `return "x" + arg0;`，恢复签名里的 `arg0` 为 `char`。

| 输入 | 原 value / 恢复 value | 原 prefixed / 恢复 prefixed |
| --- | --- | --- |
| `'A'` | `65!` / `A!` | `x65` / `xA` |
| `'0'` | `48!` / `0!` | `x48` / `x0` |
| `'中'` | `20013!` / `中!` | `x20013` / `x中` |

两侧均由 javac 编译并执行。CLI exit 0，`Complete / Java / Structured / ContainsStatements`，concat 的参数记录正确地写着 `int`、`presented=true`、`refusal=null`。结构性结果不能证明转换保值。

根因在 `crates/jarde-java/src/build.rs::concat_expr`：`ConcatPart.parameter` 已携带正确参数类型，构建只对 Boolean 做转换，其余参数原样消费；`emit.rs` 只根据首段是否 String 决定空串前缀，然后输出原表达式。`keeps_its_conversion(Int)` 证明的是 **int 作为 int 转换成字符串**，不能证明当前 Java 表达式已经是 int。`append(char)` 是否被拒绝与此无关，不需要扩大任何 overload 接受集。尚未复跑修正前二进制，因此本记录不将 T5 判作 `5c35cbf` 引入的回归。

后续最小修正：保留 `Concat { parts }`；在每个片段构建处对齐 append descriptor 要求与实际呈现表达式的类型证据，必要时保留 `(int) arg0` 一类显式转换，不能只处理首段。没有足够证据时按原契约拒绝。验收覆盖上述两种位置、char 参数/返回值/字段等已有可呈现证据、纯 int 正向对照、原 T4 行为和 T1 生命周期；不引入通用子类型系统，不混入并行导出。原主规格 `Concatenation keeps each append's own conversion` 的要求保持，不能删掉参数转换约束来把此项记作通过。

### T5 可再生复现

从仓库根目录执行；只在临时目录写输入与恢复产物，正文直接取本次 CLI 报告，未手改成预期文本。

```sh
cargo build -p jarde-cli --locked
python3 - <<'PY'
from pathlib import Path
import json, subprocess, tempfile

out = Path(tempfile.mkdtemp(prefix='jarde-append-int-'))
binary = str(Path('target/debug/jarde-cli').resolve())
(out / 'AppendIntProbe.java').write_text('''public class AppendIntProbe {
 public static String value(char c) { return new StringBuilder().append((int)c).append("!").toString(); }
 public static String prefixed(char c) { return new StringBuilder().append("x").append((int)c).toString(); }
}''')
subprocess.run(['javac', '--release', '8', '-g:none', str(out / 'AppendIntProbe.java')], check=True)
methods = []
for name in ['value', 'prefixed']:
    p = subprocess.run([binary, 'recover', '--input', str(out / 'AppendIntProbe.class'),
        '--policy', 'single-class', '--class-name', 'AppendIntProbe', '--method-name', name,
        '--descriptor', '(C)Ljava/lang/String;', '--format', 'json'], check=True, capture_output=True, text=True)
    (out / (name + '.json')).write_text(p.stdout)
    body = json.loads(p.stdout)['recovered']['recovery']['text']
    methods.append('public static String ' + name + '(char arg0) {\n' + body + '\n}')
(out / 'RecoveredAppendInt.java').write_text('public class RecoveredAppendInt {\n' + '\n'.join(methods) + '\n}')
(out / 'Runner.java').write_text('''public class Runner { public static void main(String[] args) {
 for (char c : new char[]{'A', '0', '中'}) {
  System.out.println((int)c + " value=" + AppendIntProbe.value(c) + " recovered=" + RecoveredAppendInt.value(c));
  System.out.println((int)c + " prefixed=" + AppendIntProbe.prefixed(c) + " recovered=" + RecoveredAppendInt.prefixed(c));
 }
} }''')
subprocess.run(['javac', '--release', '8', '-cp', str(out), str(out / 'RecoveredAppendInt.java'), str(out / 'Runner.java')], check=True)
subprocess.run(['java', '-cp', str(out), 'Runner'], check=True)
print('evidence:', out)
PY
```

## 保留边界与完成范围

- CLI 同一 `output_bytes` 限制同时资助请求与最终文档，发射中途停止可能无法交付报告，表现为 exit 2/stderr；库的停止和清理已通过，CLI 文档交付债务仍在，不把它说成已达到所有预算形状下的“CLI 总能返回报告”。它不推翻 T1 的无 abort 闭环。
- 字面量臂局部的类型忠实度、String/Object 的通用可赋值性、历史 clone/toString 测量归因继续独立处理；`flag() == 1` 现已可靠拒绝，不能仍记成输出非法文本。
- 默认输出限额约束长链文档大小；当前归档通过不证明未来 bulk worker 的所有栈/调度形状，bulk 仍需自己的默认 worker 栈回归。
- T5 是新的已复现保值缺口；性能调查可以继续，但正式比较必须登记该失败，不能靠删样本、更多拒绝或改变恢复质量声称加速。本轮关闭不意味着并行导出或性能专项完成。

## 第三轮历史结论（8807fa5；T1–T4 原判据现已关闭）

以下保留当时未修正的反例和结论；其中“当前”“仍开放”只指该历史构建。

## 第三轮复核：四项有独立反例的发现

代码基线为 `8807fa5bcff104584b416998c2a1d87e1f72b58a`；复核期间文档提交推进至 `98e3747`，`src/`、`crates/`、`tests/` 和 Cargo 文件无差异。当前 debug CLI 摘要为 `a76037a3cc26dc031804e9efd747b4c9c1b67d24eff1530c9a20eb908a8a67e3`。用 `git archive fa6dc6e` 导出的独立源码与独立 target 构建 boolean 修复前的 CLI，未替换工作区二进制。以下全部使用可由 javac 编译的受控源码，不依赖损坏 class。

| 编号 | 判定 | 当前结果 | 与修复前关系 |
| --- | --- | --- | --- |
| T1 / P0 | 深拼接在 **debug 构建**仍导致进程 abort | 2048 次 `.append(s)`：空 stdout、SIGABRT（Python `-6` / shell `134`），默认与放大预算一致；1024 次对照完成 | `fa6dc6e` 同样 abort，属于之前登记的深表达式未知面现已被反例证实，不归因于 boolean/array 修复 |
| T2 / P1 | 整数二元比较被错误 boolean 化 | `1 == n` → `true == arg0`；`0 < n` → `false < arg0`；`1 < n` → `true < arg0`；均 Complete/Structured，javac 拒绝 | `fa6dc6e` 三条均保持整数文本，**5a8c36a 新增回归** |
| T3 / P1 | 提升声明丢掉已声明 boolean 局部的证据 | 提升为 `int local3` 后赋入 `boolean local2`/boolean 参数，再输出 `if (local3 != 0)`；Complete/Structured，javac 拒绝 | 原路径也有类型错误；本次就地声明改为 boolean 后，提升路径仍未完成同一证据闭环 |
| T4 / P1 | concat 在数值前缀上丢失 String 转换 | `append(a).append(b).append("!")` → `a + b + "!"`；两侧均可编译，输入 `(1,2)` 原值 `"12!"`、恢复值 `"3!"` | `fa6dc6e` 也存在，属于独立 concat 语义债务，不是 receiver 括号修复的回归 |

### T1：深拼接的崩溃点已定位

生成源码：`public class DeepConcat2048 { public static String value(String s) { return new StringBuilder()` + `.append(s)` 重复 2048 次 + `.toString(); } }`。以 `javac -J-Xss32m --release 8 -g:none` 编译；JVM 执行 `value("x").length()` 正常返回 **2048**。class 摘要为 `a5b32735f840e30ec4ebad832109f57e7d8800a1eca32342084511d98cd1cb45`。

CLI 请求为 `recover --input DeepConcat2048.class --policy single-class --class-name DeepConcat2048 --method-name value --descriptor '(Ljava/lang/String;)Ljava/lang/String;' --format json`。LLDB 在 `emit.rs:476` 的左操作数打印处捕获栈访问异常，调用栈反复为 `expr → node → expr closure → binary_operand → operand → expr`；本次崩溃发生在发射，不是递归 Drop。`build.rs::concat_expr` 循环构造长度随 append 数增长的左深树，每个叶子单独通过 `MAX_VALUE_DEPTH`，因此值递归界和 region 界都约束不了树的打印深度。

**构建边界须保留**：同一当前源码的 optimized release CLI 在主线程上对 2048、4096、8192、15000 次 append 均完成；不能把 debug 的阈值说成 release 阈值。反之，release 对这些输入完成也不证明任意线程栈和 AST 形状都满足递归停止契约。修正须覆盖构建、发射及释放的实际深度，不能只加打印守卫后遗漏已有深树的清理。

### T2：boolean 转换发生在确认比较上下文之前

```java
public static int oneFirst(int n) { if (1 == n) return 3; return 4; }
public static int zeroFirst(int n) { if (0 < n) return 3; return 4; }
public static int oneLess(int n) { if (1 < n) return 3; return 4; }
```

`build.rs::condition` 在判定 `Test::Zero/Pair` 前，对左操作数调用包含 `boolean_literal` 的 `boolean_value`，再无条件按该结果调用 `boolean_spelling`。`if_icmp*` 的两个 int 操作数没有 boolean 上下文，左侧 0/1 不能据此变成 false/true。应把上下文判断置于转换之前，并覆盖常量在左/右、相等/大小比较的对照；不能以只测试 `n != 0` 证明“int 对照不变”。

### T3：就地声明与提升声明没有共享完整证据

```java
public static int hoisted(boolean b, int n) {
    boolean a = b;
    boolean c;
    if (n == 0) c = a; else c = b;
    if (c) return 1;
    return 0;
}
```

`declarations()` 的首写证据只经 `boolean_proof`，尚不认识就地声明的 boolean 局部，于是把 `c` 提升为 int。构建阶段虽然已把 `a` 声明为 boolean，`write_statement` 遇到既有 int 声明仍直接赋入 boolean，没有校验或修正。这证明“boolean 值写进 int 变量的形态不会由 javac 产生”不能作为忽略此路径的依据：int 类型来自恢复层的错误提升决策。修正应让提升/就地路径共享一致的有限证据，或在不能完成证明时可靠拒绝整个受影响结构，不能继续发布类型矛盾的赋值。

### T4：保留加法树不等于保留 append 的转换

```java
public static String value(int a, int b) {
    return new StringBuilder().append(a).append(b).append("!").toString();
}
```

`concat::plan` 接受各 append overload 后，`build.rs::concat_expr` 直接从第一个原始值开始左折叠 Binary::Add，没有确保首个 `+` 已进入字符串拼接。前两个 int 被加成数值；后续字符串只会转换求和结果。应按 append 的转换语义构造表达式或保留调用，不以调整括号修补此问题。该项与 T1 共享输入形状，但生命周期界和转换语义应分别验收。

### 已关闭项复测、门禁和证据

- 真实 `CipherSuite.isSCSV` 已输出 `true/false`，`JcaJceUtils.getDigestAlgName` 已输出 `if (…equals(arg0))`，`DSTU4145Signer.hash2FieldElement` 已输出 `byte[] local2`。
- `(a + b).substring(1)` 的 receiver 分组正确，`Mod.inverse32` 保留四次乘减分组。原 `CodeAnalyzer`、`StringUtils.stripFront` 崩溃样例本轮抽检仍返回 exit 4 的停止报告；不把这些旧反例重开。
- 独立门禁：**1285 passed / 0 failed / 6 ignored**，两项显式 JDK 对照通过；fmt/clippy 通过；OpenSpec all **19/19**、archived **23/23**。这些计数复核了用户报告，但不覆盖本轮新增形状。
- 原始证据在 `/tmp/jarde-third-review/`：`AdjacentProbe.java` 和逐方法 JSON/javac stderr；`ConcatProbe.java`、`RecoveredConcat.java`、`concat-behavior.json`；`deep-concat-summary.json`、`deep-large-budget.json`、`deep-concat-backtrace.log`、`deep-original-execution.json`、`release-depth-summary.json`；`before-boolean/` 保存独立旧版本对照。该目录是本机复核产物，不作为永久 CI 依赖。

本轮只更新审计与路线，不修改 Rust，不执行修复、不归档。主 spec 的整数上下文、boolean 证据、concat 转换和递归停止要求保留；上述四项按各自反例闭环，不能因已有 14 项变异测试就外推为所有表达式形状通过。

## 第二轮历史结论（a4dcd96；随后原反例已修正）

以下原始复核保留当时状态。receiver、boolean 返回/调用条件、数组声明原反例已由后续 `fa6dc6e`、`5a8c36a`、`66bd2d0` 分别修正；第三轮发现见上文。

## Benchmark 缺陷复测（a4dcd96，文档提交推进至 b5557f6）

本次 CLI 从 `a4dcd966af97a590ddb5787865ac89bf6b9e6645` 构建。复核过程中另一个任务完成 `2c3599a` 的契约归档及 `b5557f6` 的 README/支持矩阵提交；与构建基线在 `src/`、`crates/`、`tests/`、Cargo 文件上没有差异。本复核保留这些并行工作，只调整现有规划/文档，不修改 Rust，不提交或归档。

| 用户项 | 判定 | 当前证据 |
| --- | --- | --- |
| P0-1 三个方法导致进程 abort | **指定反例已关闭** | s2-005/007/008/009/012/013/015 的 `CodeAnalyzer.computeMaxStack()I` 均 exit 4，返回 `Partial / NotProduced`，`jre_recursion_reentry` 在 BCI 37；s2-009 的 `stripFront`、`stripBack(String,String)` 同样返回报告，位置 BCI 0。S2-008 的完整 WAR、显式 nested root、默认及放大预算也均返回报告，无 SIGABRT |
| P0-2 `Mod.inverse32` | **原反例已关闭，不能推广为所有表达式已正确** | 真实 bcprov 1.52 产物保留四次 `local1 * (2 - arg0 * local1)`；重编译后 `-1` 返回 `-1`。乘减与同级右结合的既有执行回归通过；下述 receiver 反例仍失败 |
| P1-1 boolean/int | **未修复** | `CipherSuite.isSCSV(I)Z` 仍为 `return 1/0`；`JcaJceUtils.getDigestAlgName` 仍为 `equals(arg0) != 0`。分别套入正确方法签名，javac 报 int 不能转 boolean、boolean 与 int 不可比较 |
| P1-2 数组类型 | **未修复** | `DSTU4145Signer.hash2FieldElement` 仍输出 `[B local2 = reverse(arg1)`；javac 在 `[B` 报非法表达式开头 |
| P1-3 数组 receiver | **所举具体例子归因不成立，其余条目待核实** | S2-009 的 `ArrayUtils.removeElement([DD)[D` 在 BCI 12 是 `invokestatic #149 clone:([D)[D`，不是数组的 invokevirtual。补齐同类静态 helper 后原产物编译通过；独立 `double[].clone()` 受控样例正确恢复为 `arg0.clone()`。不能据此确认所有 10 条 clone/28 条 toString 历史记录 |
| P2-1 Produced 口径 | **契约/呈现已修复，Produced 本身有意不改** | `report.rs` 明确 Produced 仅表示交付；主 spec 要求 content 独立分类；任务 CLI 的 `presentation.content` 默认可见。`clarify-structural-output-planes` 已归档 11/11。30,452/182,883 是历史统计，不是本次重测结果 |
| P2-2 重复枚举/物化 | **部分能力已交付，整体未完成** | 定向访问与有界保留已交付，但 `Budget::new` 仍 `facts: None`；默认路径未取得跨请求保留收益。本次名称选择的 plain-jar CLI 请求 bcprov 单方法计费 12,265 archive_entries，S2-008 artifact-tree scope 请求计费 4,131；它们包含名称发现，与历史物理方法请求不能直接比较。性能专项仍 0/22 |

P0-1 关闭的是进程崩溃：目前这些方法没有恢复出 Java 正文，返回明确停止是本项的预期闭环。此处没有重跑 182,883 方法的全量 sweep，也没有证明任意深度输入、所有递归构建/打印/Drop 路径都不会耗尽栈；归档中记录的深表达式边界不因本次特定反例通过而消失。

### 新确认：调用接收者的分组仍丢失

受控 Java 8 源码：

```java
public class ReceiverProbe {
    public static String tail(String a, String b) {
        return (a + b).substring(1);
    }
}
```

以 `javac --release 8` 编译，再用 `jarde-cli recover --input ReceiverProbe.class --policy single-class --class-name ReceiverProbe --method-name tail --descriptor '(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;' --format json` 恢复，正文为：

```java
return arg0 + arg1.substring(1);
```

原 class 与恢复正文分别在正确签名中编译执行，输入 `("a", "bc")`，原值为 `"bc"`、恢复值为 `"ac"`。CLI exit 0，报告 `Complete / Structured / ContainsStatements`，没有拒绝；恢复侧的 syntax/compile/semantic/verification 分别仍是 Unchecked/NotAttempted/Unproven/NotPerformed，不能称为语义验证全绿。另一个 `(a + b).length()` 样例被写成 `a + b.length()`，连返回类型也不成立。

根因位置为 `crates/jarde-java/src/emit.rs::Emitter::expr` 的 Call 分支（直接打印 receiver 后追加 `.`）。`binary_operand` 只接到二元父节点；二元子式内部正确，不等于它在调用接收者位置已有括号。`445a277` 对二元父/子分组的修复有效，但不能覆盖这个上下文；归档 verification 的“字段 receiver 等也覆盖”不能作完整分组证据。本项应独立补齐上下文分组及受控执行回归，不以扩大 `binary_operand` 的成功声明替代验证。

另外两处根因仍可直接定位：`build.rs` Return 分支未使用返回 descriptor 定型；`condition` 的 boolean 特判只识别参数，无法识别调用的 boolean 结果；`value_type` 经 `spell_reference` 只去掉对象 descriptor 包装并替换 `/`，数组 descriptor 原样泄漏。这些修正分别以返回/条件/比较和数组类型为最小闭环，不扩展成全局类型系统重写。

### 门禁、证据与下一步

同一生产代码上的独立复核：fmt/clippy 通过；固定 seed `5350648285461741569` 的 workspace 测试 **1258 passed / 0 failed / 6 ignored**；显式 JDK 执行比较 **2 passed**。这些测试尚未覆盖上述 receiver 和类型反例，不能据此关闭它们。OpenSpec 的 strict validation 仅验证规划格式。

本地原始证据在 `/tmp/jarde-second-review/`：`summary.json` 保存真实 class 命令、退出码、报告和正文，`inputs.json` 保存首批输入来源与摘要，`005-CodeAnalyzer-computeMaxStack.json` 补 s2-005；`war-default.json`、`war-large.json` 保存完整 WAR 请求；`ArrayUtils.javap`、`compile/*.stderr` 与 `receiver-behavior.json` 保存字节码、编译和行为对照。该目录不是 CI 依赖；receiver 的可再生源码已在上文保留。工作区外的真实输入来自 vulhub 的对应 WAR 与 `weblogic/weak_password/decrypt/lib/bcprov-jdk15on-152.jar`，没有运行其应用代码；只执行受控 probe 与恢复后的纯算术正文。

修复顺序：先关闭可编译但算错的 receiver 分组，再分别关闭 boolean 上下文与数组类型；逐条核实历史 receiver 报错的 opcode 与包装器上下文。主 `java8-recovery` 的保值/分组要求保持，已知实现缺口在路线中明确登记，不降低 spec。性能调查可继续，正式收益对照须固定版本、请求形状、已知缺口及保留策略；正确性实现不混入性能专项。

## 历史复核（bafdcec；R1/R2 已关闭）

以下保留当时的反例、门禁与修正补记。“当前”“仍开放”仅指该历史提交，不代表本次复核结论。

## 范围与证据基线

- 审查 HEAD：`bafdcecd36dfd7a71e2c28a4eb43c4097ff61bff`；最后行为提交 `85828c4ae2b3ebf1e6d2dc9c822ee768bbf002a2`。开始时工作树干净；本次只改规划/文档，不修改生产实现，不提交或归档。
- 重点审查近期导航、任务库操作与 CLI 的身份交接、选择和停止传播；对照主 specs、归档验证、当前性能计划及源码。全量门禁覆盖 workspace，但不宣称穷尽所有 JVM 输入与算法。
- Atlas 打开成功，但当前事实库查询新方法为空，目录查询报 `failed to validate type ranges for src/call_context.rs`；未重建全库，以下行为结论来自当前源码与实际二进制反例。
- 反例通过 `cargo build -p jarde-cli --locked` 构建的 CLI 验证，其 JSON 是库报告直接序列化；仅解析已提交受控 fixture，不执行目标代码。

## R1 / P1：不完整名称搜索被当作唯一或缺失

位置：`src/facade.rs::bind_method`（3842–3864）、`bind_class`（3732–3758）；共享搜索为 `search_named_classes`。这是 `2428752` 新任务操作路径的问题，不重开旧 R8/R9。

`bind_method` 根据 `report.candidates.len()` 直接决定未找到/执行/歧义。零和单候选分支均丢弃搜索的 execution、coverage 与 diagnostics；候选集只是已确认前缀，不能证明唯一或缺失。`bind_class` 零候选也误报缺失，单候选虽保留搜索停止，仍允许后续 body 工作。

受控 JAR 用已提交的 `NestedEval.class`，加一个字节为 `broken` 的同名路径候选；请求 `nestedPlain(I)I`：

| 输入/顺序 | 当前观察 | 应有结果 |
| --- | --- | --- |
| 仅有效 `NestedEval.class` | recover exit 0、Complete、1 body | 完整唯一的正向对照 |
| 有效 `NestedEval.class` → 损坏 `x/NestedEval.class` | recover exit **0**、`performed`、Complete、1 body；无候选损坏诊断 | 未完成选择，保留候选与损坏证据，不执行 body，CLI exit 4 |
| 损坏 `NestedEval.class` → 有效 `x/NestedEval.class` | recover/class-view exit **2**、`operation_target_not_found`；后者称只检查 1/2 候选 | 未完成选择，不宣称不存在，CLI exit 4 |

对第二行运行 class-view，可看到 `classfile_decode` 失败诊断与 Failed 搜索，证明损坏是实际发生且可报告的；recover 则丢掉该证据。仅给后续分析补一个状态不够，唯一性未证实前不能自动执行。

## R2 / P2：类视图顶层忽略方法体停止

位置：`src/facade.rs::class_view`（843–866）；`ClassViewReport` 文档承诺汇总 every body-level stop（2879–2881），实际只把 body 压入集合。`crates/jarde-cli/src/task.rs::class_view_plane`（1026 起）另行遍历补算，所以 CLI 已退出 4，直接库调用仍读到顶层 Complete。

将 fixture 的 `nestedPlain(I)I` 首 opcode（class offset 311）由 `0x1a` 改为非法 `0xff`，同时请求 `nestedPlain(I)I` 与未损坏的 `nestedLocal(I)I`：

- 顶层 `execution.status = complete`。
- 第一个 body 为 `Read`、execution Partial，`stopped_at = instructions / BCI 0 / classfile_instruction_decode`；第二个 body Complete。
- CLI exit 4；类/成员的 `artifact_structural` coverage 为 `complete_within_schema`，该结构覆盖本身合理，不应为了汇总 body 停止而改写为未扫描。

应由库汇总顶层 execution，同时保留正常 body 和每个平面的真实覆盖。已归档 CLI verification 曾披露此边界；披露不能替代库契约的闭环。

## 最小复现

从仓库根目录运行；只创建临时文件。固定 fixture SHA-256，损坏 offset 也作前置断言。输出打印退出状态与完整/停止摘要，原始 stdout/stderr 保存在显示的临时目录中。

```sh
cargo build -p jarde-cli --locked
python3 - <<'PY'
from pathlib import Path
import hashlib, json, subprocess, tempfile, zipfile

out = Path(tempfile.mkdtemp(prefix='jarde-review-'))
fixture = Path('tests/fixtures/p3-nested-eval/v8/NestedEval.class').read_bytes()
assert hashlib.sha256(fixture).hexdigest() == '141c3dcb3990605466dd54eb1a9bcc1942d0907e4e32d8d0587eb443f0c5c2e9'

def run(label, path, args):
    p = subprocess.run(['target/debug/jarde-cli', *args, '--input', str(path),
                        '--format', 'json'], capture_output=True, text=True)
    (out / (label + '.stdout.json')).write_text(p.stdout)
    (out / (label + '.stderr.json')).write_text(p.stderr)
    d = json.loads(p.stdout or p.stderr)
    execution = d.get('presentation', {}).get('execution', d.get('execution', {}))
    print(label, 'exit', p.returncode, 'execution', execution.get('status'),
          'error', d.get('error', {}).get('code'),
          'bodies', [(b['kind'], b.get('execution', {}).get('status'))
                     for b in d.get('bodies', [])])

recover = ['recover', '--class-name', 'NestedEval', '--method-name', 'nestedPlain',
           '--descriptor', '(I)I', '--policy', 'plain-jar']
for label, entries in [
    ('valid', [('NestedEval.class', fixture)]),
    ('bad_after', [('NestedEval.class', fixture), ('x/NestedEval.class', b'broken')]),
    ('bad_before', [('NestedEval.class', b'broken'), ('x/NestedEval.class', fixture)])]:
    path = out / (label + '.jar')
    with zipfile.ZipFile(path, 'w') as z:
        for name, data in entries:
            z.writestr(zipfile.ZipInfo(name), data)
    run(label + '-recover', path, recover)
    run(label + '-view', path, ['class-view', '--class-name', 'NestedEval'])

bad = bytearray(fixture)
assert bad[311] == 0x1a
bad[311] = 0xff
path = out / 'bad-body.class'
path.write_bytes(bad)
run('bad-body', path, ['class-view', '--class-name', 'NestedEval',
                      '--body', 'nestedPlain(I)I', '--body', 'nestedLocal(I)I'])
print('raw evidence:', out)
PY
```

## 门禁与完成判定

本轮在上述固定 HEAD 的代码上执行（后续工作树只改规划/文档）：

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --workspace --all-targets --all-features --locked` | seeds `5350648285461741569` / `5350648285461741570` 各 **1249 passed / 0 failed / 6 ignored** |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed（本机 JDK 23） |
| `cargo test --test p5_benchmark --locked -- --ignored --exact p5_repeated_direct_baseline` | 1 passed；18 filtered out，无性能阈值断言 |
| `cargo test --test p5_container_lookup --locked -- --ignored --exact container_lookup_timings` | 1 passed；29 filtered out，不产生本专项收益结论 |
| 修改前 OpenSpec strict | active + specs 19/19、archived 16/16 通过 |
| 修改后 OpenSpec strict | active + specs **20/20**、archived **16/16** 通过 |
| 本轮文档校验 | 17 份 Markdown 的 127 个本地相对链接无断链；修正 delta 保留主规格原有 12 个 scenario，新增 7 个；`git diff --check` 通过 |

6 个 ignored 分别是 JDK25 oracle 1、P3 编译对照 2、P5 基线 1、container 计时 1、fixture 指纹再生成 1；其中 P3 和两项 P5 已按上表另行显式运行。未运行会重写 fixture 的再生成，也未在本轮重跑 JDK25 oracle、MSRV、supply-chain 或 fuzz CI。门禁日志在本机 `/tmp/jarde-review-validation-*`；以上摘要和最小反例保存在仓库，临时日志不是永久验证资产。文中的最小复现脚本已从该 Markdown 提取并实跑，得到 R1/R2 表中结果。

上面两条反例在当前代码仍成立，常规门禁全绿不关闭它们。

完成条件：修正 change 的五项任务在固定提交验证通过，R1 不再伪唯一/伪缺失，R2 的库/CLI 状态一致，相关 delta 同步并归档。之后才更新 benchmark 的当前候选；`85828c4` 保留为历史比较臂。

## 修正结果（补记，2026-09-20）

上述两条反例已由 [preserve-task-operation-stops](changes/archive/2026-09-20-preserve-task-operation-stops/tasks.md)（5/5，固定提交 `8586356`）关闭，完成条件逐条满足。复核脚本原样重跑（库层值）：

| 场景 | 复核时 | 修正后 |
| --- | --- | --- |
| 有效 fixture | `Performed`/`complete`，exit 0 | 不变（正向对照） |
| 有效 → 损坏同名候选 | `Performed`/`complete`，执行了 body，exit 0 | `Incomplete`/`failed{classfile_decode}`，candidates=1，`method_bodies=0`/IR 0，exit 4 |
| 损坏 → 有效 | `Err(operation_target_not_found)`，exit 2 | `Incomplete`/`failed`，candidates=0，exit 4 |
| 一个 body 在 BCI 0 失败（另有正常方法） | 顶层 `complete`，CLI 靠自身补算才 exit 4 | 顶层 `partial{classfile_instruction_decode}`，正常 body 与成员表 coverage 保留，exit 4 由库报告决定 |

实现要点、变异证据（M1–M4）与门禁见该 change 的归档验证记录。**本复核的原始证据与结论保持原样**，未改写为事后通过；benchmark 当前候选随之更新为 `8586356`。

## 规划校正与分开处理的债务

- O1 定向容器访问和 backing/目录保留已在 `b22ea04` 交付；缓存现为 entries + retained_bytes 双限，默认 off。尚缺专项的 W1–W5、阶段归因、独立样本与 O1–O8 最终处置，不能将 22 项自动勾完。
- 同次 driver 的类名/access flags（包括 ACC_INTERFACE）已交接；MethodParameters、InnerClasses 的完整恢复消费和完整类级源码仍是后续范围。
- class_view 已共享一次物化字节/成员列举，但 reader 的 body 解码仍会重新解析类；成员表损坏可能连带拒绝原先可读的 body。这项局部解析/隔离债务需独立设计，不混入停止传播修正。
- `references` 按符号拼写查询，尚无物理身份直接查引用入口；WAR/Boot 自动 layout policy、现代源码输出、批量/并行/持久索引均不因本次任务链归档而成为已实现。
- 支持矩阵、路线、benchmark-review 中过期状态与归档链接已随本轮校正；历史 measurements/archives 不改写为当前运行结果。
