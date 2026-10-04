# 限定外部实例分配的 null-check 惯用法版本耦合（2026-10-04，root 巡查）

**结论：已验收切片 `recover-proved-member-inner-construction`（9/9，DT-03）只识别 javac 9+ 的 `Objects.requireNonNull` null-check 惯用法，而真实 javac 8（Corretto 1.8.0_432）对同一源码发射 `Object.getClass()` 惯用法——故 `outer.new Inner(…)` 这一 Java 8 常见形在真 javac 8 产物上被响亮拒绝，其已验收 fixture 因用 javac 23 编译而对该盲区不可见。这不是静默偏离（拒绝是响亮的、整类不投影），但属**真实 Java 8 覆盖缺口**，且与 `crates/jarde-reader/src/classfile.rs:139-141` 已固化的架构原则冲突。**

固定转录见 [fixture](fixture/) 与 [results](results/)；决定性 javap 对照见 [results/javap-idiom-comparison.txt](results/javap-idiom-comparison.txt)。

## 一、决定性单变量实验（同一源码、两种 javac）

以 DT-03 的验收锚源码 `N1.java`（[fixture/N1.java](fixture/N1.java)）为唯一变量源，只用 javac 版本区分：

| 编译工具链 | null-check 惯用法 | `N1` 族渲染引注数 | `arg1.new Inner(9).total()` 是否折叠 |
| --- | --- | --- | --- |
| javac 23 `--release 8`（**DT-03 冻结 fixture 所用**） | `invokestatic Objects.requireNonNull(Object)Object` | **3** | **是**（折叠为 `return arg1.new Inner(9).total();`） |
| **真 javac 8**（Corretto 1.8.0_432，无 `--release`） | `invokevirtual Object.getClass()Class` | **18** | **否**（整方法 `not recovered`，语句被引） |

渲染转录：[results/N1-javac23-frozen-rendered.txt](results/N1-javac23-frozen-rendered.txt)（quotes=3）vs [results/N1-realjavac8-rendered.txt](results/N1-realjavac8-rendered.txt)（quotes=18）。字节码逐条对照（**唯一差别是 BCI 6 的那一条调用**）：

```text
javac 23 --release 8:            真 javac 8 (Corretto 1.8.0_432):
  0: new           N1$Inner        0: new           N1$Inner
  3: dup                           3: dup
  4: aload_1                       4: aload_1
  5: dup                           5: dup
  6: invokestatic  Objects.        6: invokevirtual Object.getClass:()Class   ← 唯一差别
       requireNonNull:(O)O
  9: pop                           9: pop
 10: bipush        9              10: bipush        9
 12: invokespecial N1$Inner.<init> 12: invokespecial N1$Inner.<init>:(LN1;I)V
 15: invokevirtual N1$Inner.total  15: invokevirtual N1$Inner.total:()I
 18: ireturn                      18: ireturn
```

**`--release 8` 不回退该惯用法**：root 实测三种组合（真 javac 8 / javac 23 `--release 8` / javac 23 默认），只有**真 javac 8** 发射 `getClass()`，两种 javac 23 组合都发射 `requireNonNull`。故这是 **JDK 9 引入的编译器行为变更**，与 `-source/-target/--release` 无关。

## 二、`java_release` 无法用作判据（架构事实，钉死修复方向）

两者的 class 文件 **major version 均为 52**（[results/javap-idiom-comparison.txt](results/javap-idiom-comparison.txt) 末段）。故既有的 `java_release` 门（`build.rs:16177/16672/25002` 等先例）**无法区分这两种产物**——不能靠"这是 Java 8 字节码所以接受 getClass"来门控。

更根本地，`crates/jarde-reader/src/classfile.rs:139-141` 已固化原则：

> "A level is a *request*, not a property of the artifact: … the answer is stated over the facts that class really carries — **never over the compiler that produced it** and never over its own `major_version`."

**现行 `requireNonNull`-only 判据正是"依据产出它的编译器"而非"该类真正携带的事实"**：它认的是 javac 9+ 对"null 检查"这一事实的**拼写**，不是"存在一个丢弃返回值的 null 检查"这一事实本身。故修复方向应是**让判据识别该事实的两种拼写**，而不是引入版本门。

## 三、判据落点（两处，均已读码核实）

1. **`crates/jarde-java/src/init.rs:985-998`**（`new@1` 的成员构造证明）：要求
   `Load{..}` → `Duplicate` → `Invoke{ kind: Static, !is_interface_reference, owner == "java/util/Objects", name == "requireNonNull", descriptor == "(Ljava/lang/Object;)Ljava/lang/Object;" }` → `pop (0x57)`，
   否则拒绝 `"the member constructor at BCI {at} lacks the contiguous local load, dup, exact requireNonNull(Object), pop check"`。
2. **`src/member_inner.rs:126-131`**（成员调用路径）：要求 `check.opcode == 0xb8`（`invokestatic`）且 CP 条目为 `MethodRef{ owner: "java/util/Objects", name: "requireNonNull", descriptor: "(Ljava/lang/Object;)Ljava/lang/Object;" }`，否则拒绝 `"member call has no exact early requireNonNull check"`。

真 javac 8 的形是 `opcode 0xb6`（`invokevirtual`）+ `MethodRef{ owner: "java/lang/Object", name: "getClass", descriptor: "()Ljava/lang/Class;" }` + `pop`——**两处判据全部不匹配**，故两处都会拒绝。

## 四、放宽的健全性（root 已用负例实验排除误折叠）

最可能推翻"接受 `getClass()`"方案的反例是：**用户源码里显式写了 `o.getClass();`（丢弃返回值）的语句，会不会被误判为 javac 插入的 null-check 而错误折叠？** root 构造 [fixture/G-explicit-getclass.java](fixture/G-explicit-getclass.java) 实测（真 javac 8）：

```text
In explicit(G o) { o.getClass(); return o.new In(); }   // 用户语句 + javac null-check
   0: aload_1
   1: invokevirtual Object.getClass:()Class      ← 用户显式语句（在 new 之前）
   4: pop
   5: new           G$In
   8: dup
   9: aload_1
  10: dup
  11: invokevirtual Object.getClass:()Class      ← javac 插入的 null-check（在分配序列内）
  14: pop
  15: invokespecial G$In.<init>:(LG;)V
  18: areturn
```

**两者的位置可区分**：用户语句在 `new` **之前**（BCI 0-4），javac 的 null-check 是分配序列内 `aload;dup;invokevirtual;pop` 的**连续四条**且紧邻 `invokespecial` 构造器（BCI 9-14）。既有判据已要求"连续且锚定在分配点"（`init.rs` 的 contiguous 检查、`member_inner.rs` 的 `bci <= pop.bci || bci >= call_bci` 区间检查），故**在同一位置约束下改认 `getClass()` 拼写不会引入误折叠**。这条负例必须冻结进实现片的验收（见下）。

另一处需保住的边界：`getClass()` 的**符号 owner 是 `java/lang/Object`**（不是被限定变量的声明类型），且 descriptor 为 `()Ljava/lang/Class;`、返回 `Class` 而非入参类型——与 `requireNonNull` 的 `(Object)Object` 不同。故"返回被丢弃"这一事实（`pop`）是两种拼写的共同锚，实现时**不得**把"返回值 == 字段类型/入参类型"当作判据的一部分。

## 五、范围（root 实测）

- **两种站点形都受影响**：分配限定符 `new N1().new Inner(3)` 与参数限定符 `outer.new Inner(9)` 在真 javac 8 下**都**发射 `getClass()` null-check（[results/javap-idiom-comparison.txt](results/javap-idiom-comparison.txt) 与 root 对 `N1.main` 的 javap 实录）。故不是"只有某一种限定符形"。
- **与站点位置无关**：root 构造 [fixture/S-expr-vs-local.java](fixture/S-expr-vs-local.java)（同一类内 `exprForm` 直返位 vs `localForm` 本地声明位）实测**两形同样被拒**，故 root 一度持有的"站点位置是判别变量"假设**已被自己的实验证伪**——判别变量是 null-check 惯用法拼写，不是站点位置。
- **root 的三次假设证伪记录（诚实登记）**：(1) 初判"限定外部实例分配整体未覆盖"→ 被冻结 N1 fixture 渲染 quotes=0 推翻（**但那是 javac 23 产物**）；(2) 再判"判别变量是站点位置（直返 vs 本地声明）"→ 被 S/V1–V4 系列实验推翻（V1 与 N1$Stat 同形仍拒）；(3) 最终由 javap 逐条比对锁定为 **null-check 惯用法拼写**。前两次误判都源于**用 javac 23 编的探针与 javac 23 编的 fixture 对照**——同一工具链的盲区互相掩盖。**教训已固化进 handoff：涉及"javac 合成惯用法"的巡查，必须用真 javac 8 与 javac 23 双腿编译同一源码做对照，单腿无法暴露版本耦合。**
- **语料现状（root 精确普查，按"dup→调用→pop 连续"形匹配而非字面 grep）**：`tests/fixtures` 全部 class 中，携带 **`requireNonNull` null-check 惯用法的类 = 8 个**（`p3-lambda-adaptation/v8/BoundNullLambdaAdaptationProbe`、`proved-java-structure/anonymous-member-base/{AnonymousMemberBase,$1,$2}`、`recover-generic-enclosing-member-call-sites/matrix/{UseGenericObject,UseGenericTyped,UsePlain,UsePlainRaw}`），携带 **`getClass` null-check 惯用法的类 = 0 个**。即**整个 `tests/fixtures` 语料对该构造只有 javac 9+ 拼写**，结构性地对真 javac 8 产物不可见——这比"少数 fixture 用错版本"更严重：CI 无法通过任何既有测试发现该缺口。
  - **root 自查纠错（诚实登记）**：本节初稿写"109 个内部类构造器类中 7 个 `requireNonNull`、6 个 `getClass`"，那是用 `grep -q requireNonNull` / `grep -q getClass` 对 javap 全文**字面计数**得出的，把用户级 `getClass()` 调用（如 `anonymous-cross-class-use/Main` 的 `if_acmpne` 引用相等比较、`enum-string-field-name/demo/Probe` 的 `getClass().getName()`）误计为 null-check 惯用法。改用"dup→调用→pop 连续四条"的精确匹配后为 8/0，且该扫描器已用已知正例（真 javac 8 编的 `N1$Stat`）自检命中，确认非扫描器 bug。**教训**：涉及字节码惯用法的普查必须按指令序列匹配，字面 grep 会同时产生假阳（用户级调用）与假阴（跨行序列）。
  - DT-03 的验收锚 `N1` 族位于 `openspec/evidence/java-syntax-2026-10-03/inner-class-folding-patrol/fixture/`（不在 `tests/fixtures` 扫描范围），root 已单独 javap 核实其为 `requireNonNull` 形（javac 23 产物）。

## 六、归属与处置

**归属**：DT-03「限定外部实例的成员构造 `a.new AA()`」（[declarations-types.md](../../jadx-feature-inventory-2026-09-27/declarations-types.md)）。已交付片：`recover-inner-class-instance-folding`（7/7，合并 `c3805b5c`）与 `recover-proved-member-inner-construction`（9/9）。**后者的 tasks 3.1 明文把判据写成"准确 `requireNonNull` 栈形"**——即该片是**按 javac 9+ 惯用法设计并验收的**，其 spec 与 fixture 都不含真 javac 8 的形。

**这不是实现者的过错，也不是回归**：该片在其 spec 钉死的判据内是正确的，其 fixture 也确实通过；缺陷是**验收锚与语料的工具链单一**（全为 javac 9+ 产物），使版本耦合不可见。与 `op` 字段名硬编码同属一个失败模式类——**"验收锚恰好覆盖不到真实产物的变化维度"**——但这次的盲区更大：它跨的是编译器版本而非标识符名。

**处置：应立项修复**（真实 Java 8 覆盖缺口 + 与已固化架构原则冲突 + 修复形状已由本巡查钉死）。MVP 范围建议：

1. 两处判据（`init.rs:985-998`、`member_inner.rs:126-131`）各**并列**增加 `getClass()` 拼写的识别，保持"连续 + 锚定分配点 + 返回值被 `pop` 丢弃"的位置约束**逐字不变**；`requireNonNull` 支**逐字保留**（javac 9+ 产物零回退）。
2. **不得**引入 `java_release`/`major_version` 门（第二节：二者同为 52，且违反 `classfile.rs:139-141` 原则）。
3. 冻结**真 javac 8 编译**的正例（`N1` 族与一个最小 `outer.new Inner()` 形）+ 第四节的显式 `getClass()` 负例（不得误折叠）+ 既有 javac 9+ fixture 的逐字节零回退。
4. 验收须**双腿**（真 javac 8 与 javac 23 `--release 8`）都通过，且 corpus 双腿扫描的差异类**只应是此前被拒的 `getClass` 形**。

**顺带登记（独立债务，不并入本片）**：本巡查还发现 `accessor.rs` 文档记载的写访问器形为 `(LC;D)V`（void），而真 javac 8 对内部类写外部私有字段发射的是**返回值形** `(LC;I)I`（`dup_x1; putfield; ireturn`，long/double 为 `dup2_x1` + `l/dreturn`）；`accessor.rs:480` 对该形**有意拒绝**（`returns.is_some()` → 拒），呈现为空 stub → 整类不可编译（响亮，非静默）。root 全量普查：`tests/fixtures` 532 个类中**含 `access$` 的类为 0**，且 `p3_accessor_edges.rs:146-149` 合成的访问器**全部是读形**——即写访问器臂**从未被任何真实 javac 产物或测试行使过**。这与 DT-03 是同一个"验收锚覆盖不到真实产物"的失败模式，须独立取证后另行立项。

> **root 自查纠错（第二处）**：本节初稿称 `recover-private-field-owner-casts` 为"未合入片"，**错误**——root 用 `git log --grep` 与 `git merge-base --is-ancestor` 核实其实现**已合入主线**（`d09f5dea` "fix(java): recover proved parent field writes and private setter helper"，改 `crates/jarde-java/src/build.rs` +250 / `field.rs` +40），其 fixture 在 `openspec/evidence/java-syntax-2026-09-27/dt29-reference-cast-audit/fixtures/private-field/`（不在 `tests/fixtures`，故 root 先前的 `ls tests/fixtures/*private*field*` 与 `grep -rl` 都落空）。初稿的错误结论来自**只查 `tests/fixtures` 与 `git log --all --grep=<change-name>`**——后者的提交标题用的是能力描述而非 change 名。故写访问器臂的独立取证**必须**把 `d09f5dea` 已交付的"private setter helper"通路读进来（它可能已覆盖部分写形），不能当作空白起点。

原 class 为行为基准：`N1` 族真 javac 8 运行输出 `10`/`7`/`13`（[fixture/](fixture/) 内 `.java` 可用 Corretto 8 重编复现）。
