# codegen 差异普查（2026-10-04，root）——为版本耦合规则提供负对照，结清"待排查候选"清单

**背景与目的**：[dual-javac-sweep](../dual-javac-sweep/README.md) 第二节由三例缺口归纳出规则——"`--release 8` 会回退 **API 表面**差异，但**不回退编译器内部 codegen**差异"——并在该节末尾列出**尚未排查**的高风险候选：`switch` 的 table/lookup 阈值与 `default` 布局、字符串 switch 的 `$SwitchMap` 合成数组、`synchronized` 的 monitor 序列、lambda 的 `altMetafactory` 与捕获形、自动装箱的 `valueOf` 缓存路径、`assert` 的 `$assertionsDisabled` 合成字段。**本普查结清该清单**，且其结果构成规则的**负对照**（既有正例也有负例，规则才可预测而非事后归纳）。

**结论：两轮共 14 个候选构造（48 个方法）的 codegen 在真 javac 8 与 javac 23 `--release 8` 下逐维一致，均无版本盲区。**故 jarde 现有的 javac-9+ 语料对这 14 个构造**是**真 javac 8 的代表，其验收结论可外推到真 Java 8 产物——这与三例缺口（TWR / null-check / 写访问器）形成对照，说明盲区是**特定构造的 codegen 属性**而非普遍现象。

**两轮结构**：首轮 6 个单类构造（[probes](probes/) `C_*.java`，[results](results/)）；第二轮 8 个构造含 3 个嵌套类形（[probes2](probes2/) `D_*.java`，[results2](results2/)），并**补测了扫描器盲区**（嵌套类整类 opcode SHA + 协变 bridge 的 `ACC_BRIDGE` 专项）。扫描器 [codegen_differential.py](codegen_differential.py)（两轮各自移植后先用已知耦合的 `TR`/`P08_twr` 自检命中才采信 SAME 结论）。决定性指纹与自检记录 [results/instruction-sha-and-synthetics.txt](results/instruction-sha-and-synthetics.txt)、[results2/invariance-evidence.txt](results2/invariance-evidence.txt)。

## 一、方法（javap-only，零 cargo 构建）

本普查**只比对 javap 输出**，不构建 jarde、不调用 CLI——因为待回答的问题是"javac 是否发射不同字节码"，这属**输入侧**事实，与 jarde 无关。故它与在飞的实现片**不竞争磁盘**（当时 agent target 21G、余 25Gi）。

每个探针用两个工具链编译（真 javac 8 = Corretto 1.8.0_432；javac 23 `--release 8`），比对四个维度：

1. **逐方法指令 opcode 序列**（`javap -p -c` 提取 `<bci>: <opcode>` 的 opcode 列）；
2. **逐方法异常表**（行数 + `any` catch-all 行数）；
3. **整类 opcode 序列的 SHA-256**（决定性单一判据）；
4. **合成产物**：class 文件数、`$assertionsDisabled` 字段、`BootstrapMethods` 属性、常量池条目数。

**扫描器自检（先于信任其零结果）**：把两个**已知版本耦合**的类（`TR`、`P08_twr`，即 TWR 巡查的探针）喂给同一扫描器：

| 类 | 方法数 | 报 DIFF 的方法 | 报 same 的方法 |
| --- | --- | --- | --- |
| `TR` | 6 | **2**（`one`、`two` — 即 TWR 方法） | 4（构造器、`use`、`main`、`close`） |
| `P08_twr` | 4 | **1**（`one` — TWR 方法） | 3 |

即扫描器**既正确报出耦合方法、又正确区分同一类内的非耦合方法**——不是"整类盲标"，故其 `SAME-CODEGEN` 判定可信。这是 handoff「验证脚手架必须先自检」纪律的直接应用（本会话 root 曾因未自检而得到过假零结果）。

## 二、结果（两轮共 14 构造全部 IDENTICAL / SAME-CODEGEN）

整类 opcode 序列 SHA-256（[results/instruction-sha-and-synthetics.txt](results/instruction-sha-and-synthetics.txt)）：

| 探针 | 构造 | 真 javac 8 SHA | javac 23 SHA | 判定 |
| --- | --- | --- | --- | --- |
| `C_switch` | 6-case `int` switch + default | `3648dcc06466675e…` | `3648dcc06466675e…` | **IDENTICAL** |
| `C_strswitch` | String switch（含 `$SwitchMap` 路径） | `d77611feab748c87…` | `d77611feab748c87…` | **IDENTICAL** |
| `C_sync` | 实例 + 静态 `synchronized` | `5ab7c794588a315b…` | `5ab7c794588a315b…` | **IDENTICAL** |
| `C_box` | 自动装箱/拆箱（含 >127 缓存边界） | `86ba62cee81da02b…` | `86ba62cee81da02b…` | **IDENTICAL** |
| `C_assert` | `assert` + `$assertionsDisabled` | `24e759c592006237…` | `24e759c592006237…` | **IDENTICAL** |
| `C_foreach` | 数组 foreach + `Iterable` foreach | `fbe90ecd2e90c565…` | `fbe90ecd2e90c565…` | **IDENTICAL** |

逐方法比对：**6 构造共 21 个方法，0 个 DIFF**（`differential-summary.txt` 的 `differing=0` ×6）。异常表逐方法一致（`C_sync` 的两个 `synchronized` 方法两腿均 2 行 / 2 条 `any`；其余构造 0 行）。合成产物一致：class 文件数两腿相同（均 1）、`$assertionsDisabled` 仅 `C_assert` 两腿各 1、`BootstrapMethods` 六构造两腿均 0。

**这六个 opcode 序列完全一致**说明 JDK 9 的 codegen 变更**没有触及**这些构造：`switch` 的 table/lookup 选择与 `$SwitchMap` 合成、`monitorenter`/`monitorexit` + `any` 行的 monitor 序列、`Integer.valueOf` 缓存路径、`assert` 的 `$assertionsDisabled` 静态字段与 clinit、以及数组/`Iterable` foreach 的迭代形，在 JDK 8 与 JDK 9+ 下是**同一套指令**。

### 第二轮普查（同日追加，8 构造 + 嵌套类补测）

首轮 6 构造都是**单类**形。第二轮补 8 个构造（[probes2](probes2/)、[results2](results2/)），其中 3 个含**嵌套类**（协变 bridge 在 `D_covariant$B`、接口 default/static 在 `D_iface$I`）：

| 探针 | 构造 | 判定 |
| --- | --- | --- |
| `D_diamond` | 菱形 `new ArrayList<>()` / 嵌套 `new HashMap<>()` | SAME |
| `D_varargs` | `int...` / `Object...` + `String.format` | SAME |
| `D_covariant` | **协变返回 + bridge**（`B.get()` 返回 `String` 覆盖 `A.get()` 返回 `Object`） | SAME |
| `D_narrow` | 复合赋值隐含收窄（`byte += 1` / `char += 2`） | SAME |
| `D_ternary` | 嵌套三元 + null 测试三元 | SAME |
| `D_iface` | 接口 `default` + `static` 方法（Java 8 特性） | SAME |
| `D_genwit` | 泛型显式类型见证 `D.<String>id(l)` | SAME |
| `D_switchfall` | switch fall-through 合并（`case 1: case 2:`） | SAME |

**8 构造共 27 方法，0 个 DIFF**（`results2/differential-summary.txt`：`methods=27`、`differing=0`、`SAME-CODEGEN`×8、`VERSION-COUPLED`×0）。扫描器移植到本轮目录后**仍先用已知耦合的 `TR` 复验**（正确报 `VERSION-COUPLED` 2/6），故 SAME 结论可信。

**协变 bridge 专项核实**（它是最高风险项——bridge 是编译器合成的、且本会话有 `recover-bridge-superclass-header-precondition` 片涉及桥隐藏）：`D_covariant$B` 两腿都含 `String get()` + `Object get()` 两个方法、`ACC_BRIDGE` 标志都在、整类 opcode SHA 逐字节相同（`7da7c5c90545…`）→ **协变返回的 bridge codegen 版本无关**。

> **扫描器第二处盲区（诚实登记，延伸第一处）**：`codegen_differential.py` 的 `diff_construct` 只对**与源文件同名的那一个 class**（`cls + ".class"`）取 `methods()` 比对，**不比对嵌套类**（`D_covariant$B.class`、`D_iface$I.class`）。故协变 bridge（在 `$B`）与接口 default/static（在 `$I`）的版本耦合性**落在扫描器盲区内**——若只看扫描器的 `SAME-CODEGEN` 会漏判。root 因此**补做了嵌套类的整类 opcode SHA 双腿比对**（[results2/invariance-evidence.txt](results2/invariance-evidence.txt)：`D_covariant$A`/`$B`、`D_iface$C`/`$I` 四个嵌套类全部 IDENTICAL）+ bridge 的 `ACC_BRIDGE` 与方法表专项核实，才得出协变 bridge 无版本耦合的结论。**这是本会话扫描器盲区的第二例**（第一例是 opcode-only 看不到 `invokedynamic` 的 BSM 参数）——共同教训：扫描器的"覆盖范围"必须与其"结论范围"匹配，凡构造可能把逻辑放进**嵌套类**或**属性**（而非源同名类的方法体）的，都要显式补测，不能只看扫描器的绿灯。

## 三、这对规则意味着什么（正例 + 负例才构成可预测规则）

[dual-javac-sweep](../dual-javac-sweep/README.md) 第二节的规则原有三个**正例**（都是耦合的：TWR、限定分配 null-check、写访问器）。本普查补上六个**负例**（都不耦合），使规则可双向预测：

| 构造类 | javac 9+ 的变化性质 | `--release 8` 回退？ | 有盲区？ | 证据 |
| --- | --- | --- | --- | --- |
| 字符串拼接 | 改用 **JDK 9 新 API** `StringConcatFactory` | **是** | 否 | dual-javac-sweep P01（两腿均 14 次 `StringBuilder`、0 次 `invokedynamic`）|
| `switch` / String switch | **无变化**（JDK 9 未改） | 不适用 | **否** | 本普查（SHA 一致）|
| `synchronized` | **无变化** | 不适用 | **否** | 本普查 |
| 自动装箱 | **无变化** | 不适用 | **否** | 本普查 |
| `assert` | **无变化** | 不适用 | **否** | 本普查 |
| foreach（数组/`Iterable`） | **无变化** | 不适用 | **否** | 本普查 |
| 菱形语法 `new ArrayList<>()` | **无变化** | 不适用 | **否** | 第二轮普查 |
| 可变参数 `int...` / `Object...` | **无变化** | 不适用 | **否** | 第二轮普查 |
| **协变返回 + bridge** | **无变化**（`ACC_BRIDGE` 桥方法两腿同形） | 不适用 | **否** | 第二轮普查（含嵌套类补测） |
| 复合赋值收窄 `byte += 1` | **无变化** | 不适用 | **否** | 第二轮普查 |
| 嵌套三元 | **无变化** | 不适用 | **否** | 第二轮普查 |
| 接口 `default`/`static` 方法 | **无变化** | 不适用 | **否** | 第二轮普查（含嵌套类补测） |
| 泛型显式类型见证 `D.<String>id(l)` | **无变化** | 不适用 | **否** | 第二轮普查 |
| switch fall-through 合并 | **无变化** | 不适用 | **否** | 第二轮普查 |
| 限定外部实例 null-check | **内部 codegen**：`Object.getClass()` → `Objects.requireNonNull`（后者自 JDK 7 即存在，非新 API） | **否** | **是** | DT-03 巡查 |
| try-with-resources | **内部 codegen**：JDK 9 重写关闭序列（去 `aconst_null` 副本与部分 `ifnull` 守卫） | **否** | **是** | TWR 巡查 |
| 写访问器 | **内部 codegen**：栈重排（`dup_x1`）表达赋值结果 | **否** | **是** | 写访问器巡查 |

**规则（可预测形式）**：盲区当且仅当 JDK 9+ 对某构造做了**不依赖新 API 的内部 codegen 变更**。据此：
- 依赖 **JDK 9 新 API** 的变化（`StringConcatFactory`）→ `--release 8` 回退 → 无盲区；
- JDK 9 **未改动**的构造（switch/synchronized/装箱/assert/foreach）→ 两腿同码 → 无盲区；
- JDK 9 改了**内部指令选择**且所用 API 早已存在（`Objects.requireNonNull` 自 JDK 7、TWR 关闭序列纯指令重排、访问器栈重排）→ `--release 8` **不回退** → **有盲区**。

**据此可预测的剩余排查方向**（root 未实测，不外推结论，仅指出按规则应优先怀疑的构造）：凡 JDK 9 起改过内部 codegen 且不依赖新 API 者。已知 JDK 9 codegen 变更线索中值得优先双腿排查的候选：lambda 的 `altMetafactory` 参数与捕获形（本普查的 `C_*` 未含 lambda，故未覆盖）、`String` 的 compact-string 相关合成、嵌套类的 nestmate 访问（JDK 11+，但 Java 8 层级下不应出现，可作为"层级请求 vs 产物事实"的对照）、以及接口私有方法（JDK 9+ 语法，Java 8 层级不适用）。**这些须各自双腿取证后才能下结论**，本普查不代其表态。

## 四、证据强度与限制（如实标注）

- **比对的是 opcode 序列，不是 class 文件字节**：SHA 覆盖 `<bci>: <opcode>` 的 opcode 列（剥离了操作数与常量池索引）。故"IDENTICAL"证明的是**指令形状一致**，不是"字节相同的 class 文件"——两腿 class 文件字节本就不同（调试信息、路径、常量池编排不同；`C_strswitch` 常量池条目 50 vs 49）。异常表（行数 + `any` 数）与合成产物属性另作独立比对，也一致。**对本问题（"jarde 面对的 codegen 形状是否随 javac 版本变化"）opcode 序列 + 异常表 + 合成属性正是恰当的证据粒度**；字节级差异（常量池编排）不改变 jarde 的判据输入。
- **lambda 已补测（本节原记"未覆盖 lambda"，root 同日补做，见 [results/lambda-attribute-comparison.txt](results/lambda-attribute-comparison.txt)）**：补测 `L1`（无捕获 / 捕获 final 局部 / 捕获 this / 多捕获+字段，10 方法）与 `L2`（多语句块体 / 语句体，6 方法），四个维度**全部一致**：(1) 逐方法 opcode 序列 16/16 same；(2) `BootstrapMethods` 属性剥离常量池索引后 IDENTICAL（两腿同为 `LambdaMetafactory.metafactory`、`altMetafactory=0`，Method arguments 逐条相同，含 `REF_invokeStatic` 无捕获形与 `REF_invokeSpecial` 捕获 this 形）；(3) `InnerClasses` 属性剥离索引后 IDENTICAL（条目数与名字/访问 kind 均同，仅 CP 索引不同）；(4) 合成 lambda 命名惯例完全一致（`lambda$capTwo$3` 等两腿逐字相同）。故 **lambda 无版本耦合盲区**。
  - **补测暴露了扫描器盲区（诚实登记，并延伸了自检纪律）**：`codegen_differential.py` 只比 **opcode 序列**，而 `invokedynamic` 无论其 BSM 索引与参数为何都是**单一 opcode**——故 lambda 的版本敏感数据（BSM 参数、捕获形、`altMetafactory` flag）落在该扫描器盲区。若只看扫描器的 `SAME-CODEGEN` 就下结论，会把"扫描器没看的维度"误当成"一致的维度"。root 因此**补做了 (2)(3)(4) 三项 `javap -v` 属性级比对**才得出 lambda 结论。这是 handoff「验证脚手架必须先自检」纪律的**延伸**：自检不仅要验已知阳性（扫描器移植后 root 用 `TR` 复验，仍正确报 2/6 方法 DIFF），还要核对**扫描器的比较维度是否覆盖该构造的版本敏感数据**——opcode 序列对 switch/synchronized/assert 等足够，对 `invokedynamic` 系构造（lambda/方法引用/字符串拼接）不足，须加属性级比对。（字符串拼接之所以在本普查外已被 dual-javac-sweep 的 P01 覆盖，是因为它用的是 `StringBuilder` 形而非 `invokedynamic`；若某构造真用 `StringConcatFactory`，同样须属性级比对。）。[dual-javac-sweep](../dual-javac-sweep/README.md) 的 P02/P03 曾测过 lambda 与方法引用，但其结论是"双腿引注相同 → 版本无关的能力缺口"（P02 数组捕获、P03 绑定方法引用），那是**呈现侧**结论；lambda 的 **codegen 侧**双腿比对仍未做。已登记为待排查。
- **单资源/双资源 TWR 不在本普查范围**：TWR 已由专项巡查覆盖（且自检时用作已知正例）。

## 五、处置

**不立 spec**（两轮共 **14 个构造、48 个方法**均无版本耦合缺口，是健康负结果）。价值有三：
1. **结清** dual-javac-sweep 第二节登记的"待排查高风险候选"清单——首轮 6 项（switch/String switch/synchronized/装箱/assert/foreach）+ 第二轮 8 项（菱形/可变参数/**协变 bridge**/复合赋值收窄/嵌套三元/接口 default+static/泛型类型见证/switch fall-through）全部排查完毕，均无盲区。
2. 为版本耦合规则补上 **14 个负例**，使其从事后归纳变为可双向预测的判据（第三节表）。lambda 的 codegen 侧比对（原列待办）**已于同日补做**（第四节），结论为 lambda 无版本耦合。
3. **暴露并登记了扫描器的两处覆盖盲区**，均已补测：(a) opcode-only 扫描看不到 `invokedynamic` 系构造的 BSM 参数（第四节，lambda 补做 `BootstrapMethods`/`InnerClasses`/合成命名三项属性级比对）；(b) 扫描器只比对与源文件同名的那一个 class、**不比对嵌套类**（第二节第二轮，协变 bridge 在 `$B`、接口 default/static 在 `$I`，故补做四个嵌套类的整类 opcode SHA + `ACC_BRIDGE` 专项核实）。**共同教训（已固化进 handoff 脚手架纪律）**：扫描器的"覆盖范围"必须与其"结论范围"匹配——凡构造可能把逻辑放进嵌套类、属性（而非常量池索引无关的 opcode 序列）的，都要显式补测，不能只看扫描器的绿灯；且扫描器移植到新目录后须先用**已知正例**复验判别力（两轮都先用已知耦合的 `TR`/`P08_twr` 自检命中，再采信其 SAME 结论）。

**剩余未排查方向**（root 未实测，不外推；见 dual-javac-sweep 第三节末）：compact-string（JDK 9 的 `String` 内部表示，但那是 `String` 类内部、不影响调用方 codegen，风险低）、nestmate 访问（JDK 11+，Java 8 层级下不应出现，可作"层级请求 vs 产物事实"对照）、接口私有方法（JDK 9+ 语法，Java 8 层级不适用）。这些须各自双腿取证后才能下结论。
