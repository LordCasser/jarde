# 枚举双构造器委托拒绝控制

这组证据为 [recover-proved-enum-constructor-delegation task 1.3](../../../../changes/recover-proved-enum-constructor-delegation/tasks.md) 补齐负例。它以同目录父级正例的三份 Java 源码为基线；`replay.py` 在临时目录生成受控 Java 变体，或在 `javac --release 8 -g:none` 生成的 class 上做等长 opcode/常量池引用改写。每例保留 class SHA、`javap -v -c -p`、原/JADX `-Xverify:all` 输出、JADX enum 完整类、冻结 Jarde enum 类源码和 Java 8 编译结果。所有 class、jar、JADX 工作树、报告和 class 缓存都在 `TemporaryDirectory` 中，脚本退出时清理；仓库只留文本、JSON 与小型生成脚本。

从任意目录重放：

```sh
python3 /Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-25/enum-constructor-delegation/negative-controls/replay.py
```

固定工具见 `tool-versions.json`；Jarde CLI SHA 与父级正例一致：`ca04265a4f412d59c29d6bd4a26b7d9cb961f72ae13e77684831c0b9e57b5145`。九例的 `java -Xverify:all` 都成功完成。exception-handler 例使 helper 对 ZERO 抛出 `IllegalStateException`，由构造器中的合法 handler 捕获并压制，因而 handler 路径也被实际执行。name-not-forwarded 例把 `aload_1` 换成可赋给 String 的 `aconst_null`；这段字节码合法且可运行，实测 `ZERO.name()` 为 null，`Enum.<init>` 不会拒绝 null。

## 逐例结果

| 控制 | `DelegatingEnum` 的真实 BCI/结构 | 原 class `-Xverify:all` 观察 | JADX 1.5.6 重编观察 | 证明门预期 |
| --- | --- | --- | --- | --- |
| `wrong-target-long` | 无参委托 BCI 3 压入 `lconst_0`，BCI 4 调 `(String,int,long)`；另有 `(String,int)` 与 `(String,int,int)` | 值仍为 `0,1`；反射构造参数数目 `2,3,3` | 与原行为相同 | 基础唯一单 int ctor 门拒绝；委托门拒绝错误 descriptor/额外构造器 |
| `extra-int-argument` | BCI 3 为 `iconst_0`，BCI 4 为 `bipush 7`，BCI 6 调 `(String,int,int,int)` | ZERO 的字段为 7；反射数目 `2,3,4` | 与原行为相同 | 基础门拒绝额外重载；委托门拒绝额外源 int 与目标 descriptor |
| `wrong-delegate-constant` | BCI 3 `bipush 7`，BCI 5 调唯一 `(String,int,int)` | ZERO 字段与副作用实参为 7 | 与原行为相同 | 基础双构造器仍未准入；委托门拒绝实参不等于 0 |
| `delegate-argument-effect` | BCI 3 调 `ConstructorEffects.delegateValue()`，BCI 6 委托到唯一整数构造器 | 事件为 `8,0,1`，证明调用在委托前发生 | 与原行为相同 | 基础双构造器门拒绝；委托门拒绝非常量实参与额外前置效果 |
| `delegate-post-effect` | BCI 4 完成 `this(...)`，BCI 7 压入 9、BCI 9 调 helper | 事件为 `0,9,1`，证明委托后调用保留 | 与原行为相同 | 基础双构造器门拒绝；委托门拒绝委托构造器额外后置效果 |
| `exception-handler` | 终端构造器 BCI `[6,10)` 受 RuntimeException handler 保护，目标 BCI 13；StackMapTable 覆盖该 handler | 捕获并压制 helper 为 ZERO 抛出的异常，正常完成，值仍为 `0,1` | 与原行为相同 | 基础门不接受；委托门拒绝异常边及 handler/helper 的额外行为 |
| `name-not-forwarded` | 无参构造器 BCI 1 为 `aconst_null`，取代原 name 参数；其余仍在 BCI 4 委托 | ZERO 名字变为 null，ordinal 仍为 0 | JADX 重编把它规范化回 `ZERO`，丢失原 null name | 基础门不接受；委托门必须证明原 name 的参数来源，不接受 null/猜名 |
| `ordinal-not-forwarded` | 无参构造器 BCI 2 为 `iconst_1`，取代原 ordinal 参数；BCI 4 委托 | ZERO ordinal 变为 1，与 ONE 重复 | JADX 重编把它规范化回 ZERO ordinal 0 | 基础门不接受；委托门必须证明原 ordinal 的参数来源 |
| `values-helper-order` | `$values()` BCI 6 读取 ONE，BCI 12 读取 ZERO；`<clinit>` 常量字段建立顺序未变 | `values()` 为 `ONE,ZERO`，其真实 ordinal 仍是 `1,0` | JADX 改用 `ONE,ZERO` 声明；重编 ordinal 变成 `0,1`，构造效果顺序也变成 `1,0` | 基础枚举整体门先拒绝隐式数组/helper 不一致，不能进入委托投影 |

每个表行对应一个目录；其中 `class-sha256.json` 列出该例所有 class 的 SHA，`javap.txt` 是涵盖私有构造器与 helper 的完整反汇编，`result.json` 汇总三方退出码、原/JADX 观察和预计拒绝理由。JADX 为九例全部生成可编译 Java；只有 name、ordinal 和 `$values` 三个反例显示其源码重编会归一化掉原 class 中的反常行为。冻结 Jarde 对九例均返回完整 class-source 报告，但依旧把 enum 常量写成普通静态字段；Java 8 编译均以 exit 1 在第一条字段声明处失败，所以没有 Jarde 重编类可运行。各目录的 `jarde-DelegatingEnum.java.txt` 与 `jarde-javac.txt` 保留该对照。

这些都是可运行 classfile 控制，不存在只在证明函数层面的样例。除其表中标出的单个关系/效果改写外，脚本生成的 classfile 仍由原 Java 8 源或等长 opcode 替换导出；SHA 与 BCI 使每个控制可复核。JADX 的错误修复也说明它只能作为文本对照，不能用作接受或拒绝的正确性依据。当前 Jarde 无法产出任何 enum 可重编文本，故本证据记录的是基线拒绝，不声称已经测试新证明器。

Root 从仓库外的临时目录复制整个父级证据目录后运行 `negative-controls/replay.py`，没有改动冻结文件；九例均完成，原/JADX 的 Java 8 编译和 `-Xverify:all` 退出码均为 0，冻结 Jarde 的 Java 8 编译退出码均为 1。重放的 `summary.json` 除运行时长 `elapsed_millis` 外与冻结摘要逐字段相同；临时 class、jar 和输出在退出时清理。
