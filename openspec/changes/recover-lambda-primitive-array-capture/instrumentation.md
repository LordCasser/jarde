# 插桩定夺（task 1.2）与基线重验（task 1.3）——方向 A/B

实测跨两次合并态：
- 初测：worktree HEAD `2d959805`（= 当时的 `origin/main`，`git merge origin/main` → `Already up to date`）。
- **终测（本文件数字的权威）**：合并到 `origin/main` = `54a64a57`（其间主线只有 evidence/docs 提交，
  `git diff --name-only 2d959805..54a64a57` 对 `crates/ src/ tests/ fuzz/` 为 **0 个文件**），
  再在同一树上用 `cargo build --workspace --locked` 分别产出「未改动主线二进制」与「本片改动二进制」对照。

构建产物：`target/debug/jarde-cli`（渲染姿态 `class-source --policy plain-jar --release 8 --format text`；
实测该 stdout 与库内 `report.text` 逐字节相同——与 EM-15 冻结记录 `fixed/WA-rendered-fixed.txt` 对拍通过）。

## 1.2(a) P02 class 是否含 LocalVariableTable —— **决定性：双腿都不含**

`javap -v -p` 实测（真 javac 8 = corretto-1.8.0_432；javac 23 = openjdk-23.0.1 `--release 8`）：

```
=== a8 (real javac 8) attribute tables present ===
   2 BootstrapMethods
   2 InnerClasses
   7 LineNumberTable
   2 SourceFile
=== b23 (javac 23 --release 8) attribute tables present ===
   2 BootstrapMethods
   2 InnerClasses
   7 LineNumberTable
   2 SourceFile
```

`grep -c LocalVariableTable` = **0**（双腿）。design.md Open Question 3 的高风险分支成立：
**冻结 fixture 按 javac 默认 `-g:lines,source` 编译，不发 LocalVariableTable**。

对照腿（仅作信息源证明，**不是**验收输入）：同一源用 `-g` 全量调试信息重编，LVT 出现 7 处，`sum` 表项含
`t [I`。即"名字表陈述了 `[I`"这一前提在冻结 fixture 上**不成立**——以 LVT 为陈述源的 B 变体在主锚上无输入可用。

该事实已固化为断言：`tests/recover_lambda_primitive_array_capture.rs::the_frozen_anchor_is_the_recorded_patrol_probe`
对四个冻结 class 逐个断言字节流中不含 `LocalVariableTable`。

## 1.2(b) `capture_types` 的 `ty` 实际来源与传导链

拒绝文本里的 `Object` **不是名字表给的，是帧层 SSA 值给的**：

- `crates/jarde-java/src/build.rs`（改动前 23585）`Builder::lambda_expr` 的捕获类型取
  `value_type(self.ssa.value(*value).ty()).ok().flatten()`。
- `crates/jarde-java/src/build.rs:26910` `value_type`：`Ref(Named{name,..})` → `spell_reference(name)`
  （`[I` → `int[]`）；`Value::Ref(_) | Value::Null` → `Type::Reference("Object")`。
- `crates/jarde-jvm/src/frame.rs:2299-2316` `PoolEffect::NewArray` 对 `newarray` **有意**压
  `Value::Ref(RefType::Unknown)`（注释原文：bootstrap loader 未声明，故不锚名）。

传导链：`newarray`（BCI 1，atype=10）→ 帧 `Ref(Unknown)` → `value_type` → `Object` →
`lambda.rs:765` 的 `frame_type` → 与站点/实现的 `int[]` 不一致 → `jre_lambda_sam_types` 拒。
**名字表全程不参与**。

**判别实验（同形只换类型来源；改动前后都不变，用于定位而非验收）**：

| 探针 | 形 | 改前实测 | 结论 |
| --- | --- | --- | --- |
| `Diag1` | `static Consumer<Integer> g(int[] t){ return i -> t[0]+=i; }`——同一 `[I`，但捕获值来自**方法参数**（帧 `Named{[I}`） | **0 引注、捕获内联成功** | **判据本身正确**；错的只是喂给它的帧类型 |
| `Diag3` | `long[]`、`boolean[]` 捕获（atype 11/4） | 各 1 处引注（`Object` vs `long[]` / `boolean[]`） | 缺口覆盖全部十种 atype，非 `[I` 独有 |
| `Diag2` | `int[][] t={{0}}`（`multianewarray`）捕获 | **捕获站点 0 引注、已内联**；helper 体另有 5 处 | `int[][]` 捕获门现状=已放行（见 §1.2(e)，其渲染文本另有既有可编译-错行为） |

## 1.2(c) `RefType::Unknown` 全消费面清单（方向 A 的前置核对）

`grep -rn "RefType::Unknown" crates/` 的非测试产生点/消费点逐个核对：

| # | 位置 | 角色 | 方向 A（`newarray` 给名）后的行为 |
| --- | --- | --- | --- |
| P1 | `frame.rs:2316` `PoolEffect::NewArray` | 本片目标产生点 | `Unknown` → `Named{[Z..[D}` |
| P2 | `frame.rs:342` `Produced::value` | 产生点 | 不触及（`newarray` 走 P1） |
| P3 | `frame.rs:1374` `merged_reference` 合并落点 | **消费点：以 Unknown 作合并结果** | **变**：同一 `[I` 与 `[I` 的合并改由"名字+loader 相等"决定，不再无条件 `Unknown` |
| P4 | `frame.rs:1562` `value_of`（`Ty::Ref` 无名） | 产生点 | 不触及 |
| P5 | `frame.rs:2154` `ldc` String/Class/MethodType/MethodHandle | 产生点 | 不触及 |
| P6 | `frame.rs:2597` `caught_reference`（catch-all） | 产生点 | 不触及 |
| C1 | `frame_oracle.rs:437` `project`：`Unknown → OValue::Ref(None)`，`Named → Ref(Some(internal_name))` | **消费点：字节码 oracle 对照** | **变**：`[I` 被投影为 `Some("[I")`，与 oracle 该槽位期望需重新核对——独立验证面 |
| C2 | `build.rs:25564` `constant_of_value` 前置 `matches!(.., Unknown)` | **消费点** | 不变（`ldc` 走 P5，仍 Unknown） |
| C3 | `enumswitch.rs:1472` `selector_receiver_type`（`Unknown` 与非 `Ref(Named)` 同归 `None`） | **消费点** | **取值变**：`None` → `Some(b"[I")`；下游按 enum 类名比较仍失败、判定结果不变，但**消费者拿到的值变了** |
| C4 | `build.rs:25439` `array_of_value` 首判（`Named → array_descriptor`，否则读生产者） | **消费点（本片复用的通道）** | 变且有利：`[I` 给名后由"生产者读取"改"帧描述符读取"，两者答案相同 |

**停手判定**：C1（oracle 投影）、C3（`Some("[I")` vs `None`）、P3（合并不再无条件 Unknown）都是**消费者行为变化**。
任务书 task 2.2 与 design 验证标准 5 规定"任何消费者行为变化即停下报告"。方向 A 无法在不改这些消费者期望的前提下落地 → **A 否决**，本节即否决证据。

## 1.2(d) `RefType` 能否承载描述符形 `[I`

**能**：`anewarray` 经 `frame.rs:1904 array_of` 存 `[LName;`，`multianewarray` 经 `frame.rs:2320` 存 `[[I`，
`frame.rs:1395` 注释明写"An array descriptor (`[LLazy$Holder;`) is a type of its own and keeps its spelling"。
故 A 的类型格容量不是障碍；障碍在 §1.2(c) 的消费面。

## 定夺：**方向 B**——捕获门（`capture_types` 来源）取字节码陈述的类型，陈述源为 `newarray` 的 `atype` 码

按 design 决策 1 的取舍标准（"若 Unknown 消费面排查显示无依赖（A 安全）且 B 的类型陈述不可得，则 A；否则 B"）：

1. **A 不安全**：§1.2(c) 的 C1/C3/P3 消费者行为随给名变化（C1 触及 oracle 对照面）→ 停手条件命中。
2. **B 的陈述源可得，但不是 LVT**：主锚双腿无 LVT（§1.2(a)），按 Open Question 3 的预告，陈述类型必须来自
   字节码本身陈述的事实——`newarray` 的 `atype` 码。该读取**仓库已有**：
   `crates/jarde-java/src/decode.rs:487-498`（`0xbc` → `Operation::NewArray{element,1,1}`；
   `primitive_array_of`：`4→Boolean 5→Char 6→Float 7→Double 8→Byte 9→Short 10→Int 11→Long`）与
   `crates/jarde-java/src/build.rs:25386` `array_of_value`（帧 Unknown 时读**创建指令**的 atype，并穿过
   `Duplicate`/`Load`/`Store`/`ArrayElementLoad` 追值）。
   `written_type`（build.rs:25538）**早已**用这条通道把 `int[] local1 = new int[]{0};` 的声明类型写对——
   **只有 `lambda_expr` 的捕获类型没用它**。故 B 是窄修：把同一"已陈述事实"接到判据的 `frame_type` 输入上。
3. **不动帧层保守语义**，**不动** `lambda.rs` 三方判据一字（见下）。

### 落点与 proposal.md Impact 的对应

proposal.md Impact 写的是"`crates/jarde-jvm/src/frame.rs`（仅方向 A）**或** `crates/jarde-java/src/lambda.rs` 的
**`capture_types` 来源**（方向 B）+ `names.rs`（若 B 需要读局部陈述类型）"。本片实现即落在**`capture_types` 来源**：
新增 `capture_value_type`（`crates/jarde-java/src/build.rs`，紧邻 `written_type`）并让 `lambda_expr` 用它。
`names.rs` **未触及**——仓库既有的 `array_of_value` 通道已承载该陈述读取，无需新建 LVT 读取。

### 判据零放宽的证据

`capture_value_type` **只在帧陈述为 `RefType::Unknown`** 时才允许被替换，且替换值必须是
`array_of_value` 从创建指令读出的数组形；其余一切取值路径逐字不变：

- 站点/实现/帧任一不一致 → 仍拒（`a_newarray_capture_whose_implementation_disagrees_still_refuses`：
  帧 `int[]` / 站点 `int[]` / 实现 `java.lang.Object` → `jre_lambda_sam_types`）。
- 帧为 `Named` 的形 → 走改动前同一 `spell_reference` 路径。
- `Value::Null` 捕获、合并（phi/两写不同类）得到的 Unknown、无单一创建可陈述的形 → 仍 `Object`。
  `Diag4`（`if(b) t=new int[]{1}; else t=new int[]{2};` 后捕获）改前改后**都拒**：两写的 phi 值定义是
  `Definition::Phi`，`array_of_value` 的 `Definition::Instruction` 前置不成立，取不到单一陈述。
- 字节级证明：`git show HEAD:crates/jarde-java/src/lambda.rs` 与工作树中从
  `// A capture runs while the functional value is created` 到 `let adaptation = match adaptation_plan(`
  之前的整块（29 行，含判据 `frame_type != site_type || site_type != implementation_type` 与
  `jre_lambda_sam_types` 文本）对拍 **byte-identical**（sha256 前缀 `f0bba913` 两侧相同）。
  `lambda.rs` 的实际 diff 只有两处注释/doc（`plan` 的 doc、`capture_types` 上方的注释）——
  task 2.3 要求的"注释随取值来源变化同步"。

## 1.2(e) `int[][]`（multianewarray）现状如实记录

实测（改动前后一致）：`int[][] t={{0}}; l.forEach(i -> t[0][0]+=i)` 的**捕获站点从未被拒**——
`multianewarray` 在常量池点名 `[[I`，帧 `Named{[[I}` 已陈述类型，判据本就有据可比。该形渲染里 5 处引注
**全部在 companion 体内**（`t[0][0] += i` 的二维复合赋值，"not part of the provable subset"），
属 proposal.md Non-Goals 里"lambda 体内数组元素复合赋值的表达式化"这一既有域，不是捕获门。
本片对它的渲染 **byte-identical**（`baseline/` vs `fixed/`，双腿），固化为
`the_multianewarray_capture_keeps_its_recorded_status`。

### 1.2(e)-附：该 control 渲染文本的**可编译-错行为**（本片段之外，如实登记，未修）

`P02_multianewarray` 的 `fixed/` 文本（前置 `import java.util.*;` 等两行）实测：

| 检查 | 结果 |
| --- | --- |
| `javac --release 8` | **exit 0** |
| `java -Xverify:all` 渲染重编物 | 输出 **`0`** |
| `java -Xverify:all` 原 class（同一 v8 fixture） | 输出 **`6`** |

`lambda$sum$0$jarde` 体内的 5 处引注把整条 `arg0[0][0] += arg1.intValue();` 丢光，函数体只剩 `return;`——
文本能编译，但语义上不再累加。按主线 soundness-patrol 词汇这是 **compilable-wrong**（静默偏离）一类，
与本片主锚那种"响亮拒绝"方向相反。

三点必须说清：

1. **本片未改变它**：`baseline/` 与 `fixed/` 双腿逐字节相同（断言 5 即此条），引注数量同为 5。本片 diff
   只在"帧 `RefType::Unknown` 且存在单一创建陈述可读"时改变取值；`[[I` 的捕获从来不走这条路。
2. **它不是本片引入的**：属 proposal.md Non-Goals（"lambda 体内数组元素复合赋值的表达式化……捕获门放行后
   按既有路径走"）与 DT-26 既有域的交叠处。既有路径在此形的落点是"companion 体拒绝呈现该语句"，
   而捕获门**本就放行**，所以呈现出的文本带引注地丢了这条语句。
3. **建议 root 判定是否另立**：与主线已登记的 `t[0]+=i` 元素复合赋值表达式化缺口同源，但多一维
   （`[[I` 的 `aload; iaload; … iastore` 链）。本片按停手条件不扩权去修，也不改其断言来掩盖。

## 1.3 基线重验（未改动主线二进制，改动前）

| 腿 | stdout 与已提交证据逐字节 | 源码区 `@bytecode` | 引注 BCI | `map` 内联 |
| --- | --- | --- | --- | --- |
| a8（真 javac 8） | **IDENTICAL** | 1 处（同一拒绝点 3 个 BCI） | `15 8 9` | ✓ |
| b23（javac 23 `--release 8`） | **IDENTICAL** | 1 处 | `15 8 9` | ✓ |

**计数口径（需 root 复核时对齐）**：proposal/spec 写"3 引注（BCIs 15/8/9）"，实际渲染是 **1 行**
`// @bytecode 15 8 9` 标记（该 marker 一行列出三个 BCI），`dual-javac-sweep/README.md` 的表里 P02
也记作"真 javac 8 引注 1"。修复后两种口径都是 **0**（0 行 marker、0 个 BCI），故主锚"0 引注"无歧义；
改前口径不一致只影响基线描述的措辞，不影响验收。

双腿拒绝文本一致（`captured operand 0 is `Object` in the frame, `int[]` in the site descriptor and
`int[]` in the implementation: ...`）→ 本形**版本无关**，与账本登记一致。

## 改动后实测（终测，HEAD `54a64a57`）

| 项 | 结果 |
| --- | --- |
| 主锚 `P02_lambda.sum` 双腿 | 源码区 **0 引注**（含 `captured operand` 0 处），`sum` 与 `map` 同构内联 |
| 渲染文本 `javac --release 8` | 双腿 exit 0 |
| 行为 | 双腿 original 与 recompiled 均 `6\nab\n`，逐字节相同 |
| 零回退 | `map` / `lambda$map$1` / `main` / `<init>` 成员块 baseline↔fixed **byte-identical** |
| DT-26 既有锚 `CaptureCases` | 改动前后渲染 **byte-identical**；其文本重编+运行输出仍 `7:-1` |
| 语料零回退 | 590 个已提交 `.class`（含本片 4 个）双二进制对照：**只有本片 `P02_lambda` 两腿差异**，其余 588 呈现一致 |
| 十探针双 javac 扫描 | 只有 `P02_lambda` 双腿差异（各 4 行），其余 9 探针 ×2 腿逐字节 SAME |

## 未完成/限制（如实）

- `replay.py`（DT-26 固定重放）在本片改动下仍失败，**且未改动主线二进制同样失败**：其
  `require("lambda$" not in jarde_text, ...)` 断言与后续 lambda-companion 重命名 marker 文本冲突（marker 现写
  `// jarde: omitted physical lambda helper "lambda$add$0" ...`）。这是**主线既有 staleness**，与本片的
  捕获门无关；本片的真实验收（原文重编 `--release 8` + 运行 `7:-1`）已单独实测通过。不在本片修它（超出允许修改面）。
- 双 javac 扫描的 `P07_inner_outer` 与已提交 evidence 有差异，实测为**主线既有漂移**（EM-15 合并 `af6f65c4`
  在 2026-10-05，晚于 2026-10-04 的该扫描记录）：未改动主线二进制渲染 P07 与已提交记录同样差异，
  且 P07 全部 class **不含任何 invokedynamic**（本片改动的代码路径只从 `Operation::InvokeDynamic` 分支进入）。
  本片不改该 evidence。
