# try-finally 的 codegen 版本不变性（2026-10-04，root）——为版本耦合风险划定边界

**结论：`try-finally`（含 return-in-try、try/catch/finally 三层、嵌套 finally、循环与 finally 互相嵌套、多返回点）的 codegen 在真 javac 8（Corretto 1.8.0_432）与 javac 23 `--release 8` 下**完全一致**——整类 opcode 序列 SHA-256 相同、逐方法异常表逐行相同。故 [TWR 缺口](../twr-javac8-codegen-patrol/README.md) 的版本耦合**是资源关闭惯用法（resource-close idiom）独有的，不是 `finally` 机制的普遍性质**。**

这个负结果的价值在于**划定风险边界**：jarde 为 `finally` 族维护了 **13 个**定长 shape 变体（root 实测清点 `guard.rs` 的 `pub enum Shape`：**16 个变体 = 13 个 finally 族 + `Resources`（变长 TWR）+ `Monitor` + `MonitorBranches`**；13 个为 `Finally`、`LoopFinally`、`ConditionalFinally`、`NullableResourceFinally`、`FlagConditionalFinally`、`LocalNullConditionalFinally`、`SharedFinally`、`EmptyCatchCallFinally`、`TwoCatchReturnFinally`、`NestedCleanupFinally`、`SegmentedFinally`、`MultiReturnLoopFinally`、`SegmentedNullLeadFinally`），其验收 fixture 全部由 javac 9+ 编译。若 `finally` 也版本耦合，这 13 个变体的验收都会有与 TWR 同类的盲区——**实测否证了这一点**，故 finally 族的既有验收**可以外推到真 javac 8 产物**，只有 TWR（由变长的 `Shape::Resources` 承载）需要专项处理。

探针 [probes](probes/)（`F1.java` 4 形、`F2.java` 3 形）、扫描器 [finally_differential.py](finally_differential.py)、逐方法差异 [results/differential-summary.txt](results/differential-summary.txt)、决定性证据 [results/invariance-evidence.txt](results/invariance-evidence.txt)。

## 一、覆盖的 7 个 finally 形（都是 Java 8 常见形）

| 探针方法 | 源形态 | 真 javac 8 | javac 23 |
| --- | --- | --- | --- |
| `F1.plain` | `try { return s.length(); } finally { … }`（return-in-try） | 14 instrs / 1 exc / 1 any | 14 / 1 / 1 |
| `F1.noRet` | `try { … } finally { … }` + try 后 return | 15 / 1 / 1 | 15 / 1 / 1 |
| `F1.catchFin` | `try / catch(RuntimeException) / finally` 三层 | 22 / 3 / 2 | 22 / 3 / 2 |
| `F1.nested` | 嵌套 `try{ try{…}finally{…} }finally{…}` | 23 / 3 / 3 | 23 / 3 / 3 |
| `F2.loopFin` | finally **内含**循环 | 25 / 1 / 1 | 25 / 1 / 1 |
| `F2.finLoop` | 循环**内含** finally | 31 / 1 / 1 | 31 / 1 / 1 |
| `F2.multiRet` | 多返回点 + finally | 22 / 2 / 2 | 22 / 2 / 2 |

（格式：指令数 / 异常表行数 / 其中 `any` catch-all 行数；真 javac 8 与 javac 23 两列**逐值相同**。）

**11 个方法（含两类的构造器与 main）全部 `same`，0 个 DIFF。**

## 二、决定性证据（三个维度，不止指令数）

1. **整类 opcode 序列 SHA-256**（`javap -p -c` 提取全部方法的 opcode 列后求 SHA）：

   | 类 | 真 javac 8 | javac 23 | |
   | --- | --- | --- | --- |
   | `F1` | `e67dae668d75749f0672bd595e26f4a2` | `e67dae668d75749f0672bd595e26f4a2` | **IDENTICAL** |
   | `F2` | `d7a3b1572ca45cb3c91d79f71382973d` | `d7a3b1572ca45cb3c91d79f71382973d` | **IDENTICAL** |

2. **逐方法异常表逐行对照**（`finally` 的形状证明主要由异常表驱动，故这比指令数更关键）：`F1` 四方法的行全部 `SAME`，例如
   - `plain`：`0 5 15 any`
   - `catchFin`：`0 5 15 Class java/lang/RuntimeException; 0 5 28 any; 15 18 28 any`
   - `nested`：`0 5 23 any; 0 13 34 any; 23 35 34 any`

   即 **`any` catch-all 行的数量与区间在两腿完全一致**——这与 TWR 形成鲜明对照（TWR 的 `any` 行数真 javac 8 为 2/4、javac 23 为 **0**）。

3. **逐方法指令数与 `any` 计数**：11/11 相同（见上表）。

## 三、扫描器判别力自检（负结果的可信前提）

"全部 same"这类**负结果**必须先证明扫描器有判别力，否则可能只是扫描器看不见差异（root 本会话已因未自检得到过假零结果，见 handoff）。扫描器移植到本目录后，root 先用**已知版本耦合**的 `TR`（TWR 探针）复验：

```text
TR: VERSION-COUPLED (2/6 methods differ)
  [same] public TR(String)                 instrs 6->6    exc 0->0   any 0->0
  [same] public String use()               instrs 10->10  exc 0->0   any 0->0
  [same] public static void main(String[]) instrs 9->9    exc 0->0   any 0->0
  [same] public void close()               instrs 12->12  exc 0->0   any 0->0
  [DIFF] static String one(String)         instrs 22->48  exc 2->5   any 0->2
  [DIFF] static String two(String)         instrs 61->113 exc 5->11  any 0->4
```

即扫描器**既对已知耦合构造正确报警、又能区分同一类内的非耦合方法**（`close()`/`use()` 等 4 个方法正确报 same），故其对本普查 11 个方法的 `SAME-CODEGEN` 结论可信。

## 四、与 TWR 的对照：为什么 finally 不变而 TWR 变

| | try-finally | try-with-resources |
| --- | --- | --- |
| `any` catch-all 行 | 两腿**都有**且数量相同（javac 8 起 finally 一直用 `any` 行表达"任何路径都要执行"） | 真 javac 8 有 2/4 条，javac 23 **0 条** |
| 资源副本前置 | 不适用 | 真 javac 8 有 `aconst_null; astore`，javac 23 无 |
| 关闭序列副本数 | 不适用 | 真 javac 8 `one()` 4 份 close + 2 份 `addSuppressed`，javac 23 显著简化 |
| 结论 | **版本无关** | **版本耦合** |

即 JDK 9 对 TWR 的 codegen 重写**只改了"资源关闭"这套惯用法**（去掉可证非空的资源副本前置、简化守卫关闭序列、不再发 `any` 行），**没有改 `finally` 的通用 lowering**。这解释了为什么 jarde 的 finally 族（12 个定长变体）在 javac 9+ fixture 上的验收依然对真 javac 8 产物有效，而 `Shape::Resources`（变长 TWR 变体）无效。

**这条边界也修正了 [TWR 巡查](../twr-javac8-codegen-patrol/README.md) 第三节的一处过度概括**：该节曾把 TWR 的 javac 8 拓扑与 `Shape::SegmentedFinally`（"five-row, two-segment, four-copy **Java 8** finally certificate"）逐维对照，暗示二者同族。本普查证明 **finally 族的 "Java 8" 拓扑在两腿不变**，故 `SegmentedFinally` 等 finally 变体的既有验收对真 javac 8 有效；TWR 的 5 行拓扑与 `SegmentedFinally` 的 5 行**只是数字巧合，不是同族**——TWR 由 `Shape::Resources`（变长）承载，其缺口是资源关闭惯用法的版本差异，与 finally 族的定长变体无关。

## 五、处置

**不立 spec**（无缺口，是健康负结果）。价值有二：
1. **划定系统性风险的边界**：本会话发现的三例版本耦合缺口（TWR / 限定分配 null-check / 写访问器）都是**编译器为某个源构造生成的特定惯用法**发生了 JDK 9+ 变更；而 `finally`、`switch`、String switch、`synchronized`、装箱、`assert`、foreach、lambda（含 BSM 参数）经双腿实测**均版本无关**（见 [codegen-differential-sweep](../codegen-differential-sweep/README.md) 与本文件）。故"语料由 javac 9+ 编译"这一系统性问题的**实际影响面比初看起来小得多**，不需对全语料做真 javac 8 重编（那会改变全部字节 SHA、fingerprint 与所有以 SHA 断言的测试）——只需对**已证的三个惯用法**专项处理。
2. **纠正一处过度概括**（第四节末）：TWR 与 finally 族不同族，不得因行数巧合而混谈；这直接影响 TWR 后续片的落点判断（落在 `Shape::Resources` / `fn twr`，与 finally 定长变体无关）。

原 class 为行为基准（root 以 Corretto 8 的 `java` 实测真 javac 8 腿，两腿输出相同）：

```text
F1: fin1 | x | fin2 | fin3 | inner | outer | 7        （即逐行 `fin1`、`x`、`fin2`、`fin3`、`inner`、`outer`、`7`）
F2: loopfin | mid | mr | mr | 6
```

（`|` 是 root 为便于比对把换行折成的分隔符。）**实测与推断一致**：本节初稿按源码推断末行为 `2+1+2+2`（=7）与 `3+3+1-1`（=6），实测输出正是 **7** 与 **6**，故推断值成立、无需更正。**但流程上仍是错的**：本普查的验收项是**输入侧 codegen 比对**（与 jarde 呈现无关），行为值只作 fixture 基线记录——root 在**未实测**的情况下就把推断值写进了证据，若推断有误就会被后续引用为"原 class 行为"而无人察觉。本会话 root 已有两次因未实测而写错数字的记录（`two()` 的 `any` 行数、TWR 异常表项的单方法/整类口径混淆），故此处补做实测并如实标注"推断与实测一致"，而不是把它当作已实测的结论交付。实测输出已冻结于 [results/](results/)。
