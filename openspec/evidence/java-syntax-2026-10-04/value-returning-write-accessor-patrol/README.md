# 返回值形写访问器（`access$(C,D)D`）不被恢复（2026-10-04，root 巡查）——独立于 DT-03 缺口

**结论：`accessor.rs` 的写访问器判据（文档记载为 `(LC;D)V` void 形）与真实 javac 产物不符。真 javac（8 与 23 皆然）对"内部类写外部私有字段"生成的是**返回值形** `access$(C,D)D`（`aload_0; iload_1; dup_x1; putfield; ireturn`），`accessor.rs:480` 的 `if returns.is_some()` 分支**有意拒绝**该形，渲染为空 stub（`static int access$002(WA arg0, int arg1) { }`）→ 整类不可编译（响亮失败，非静默偏离）。这是独立于 [DT-03 `getClass` 缺口](../qualified-outer-alloc-getclass-patrol/README.md) 的第二个"验收锚覆盖不到真实产物"缺陷。**

固定转录见 [fixture](fixture/)（`WA.java`/`WA.class`/`WA$Setter.class`、`I1.java` + 4 class）与 [results](results/)（javap 形对照、两份家族渲染）。

## 一、决定性证据：写访问器在真 javac 产物中是返回值形

`accessor.rs` 文档（第 13-14 行）记载两种形：

```text
read:   load 0 ─ getfield C.f:D ─ return          descriptor (LC;)D
write:  load 0 ─ load 1 ─ putfield C.f:D ─ return descriptor (LC;D)V     ← void
```

但真 javac 对 `v = x`（内部类写外部私有字段）生成的是**返回值形**，不是 void：

```text
WA (javac 23 --release 8):  static int access$002(WA, int)     descriptor (LWA;I)I   ← 返回 int，非 void
   0: aload_0
   1: iload_1
   2: dup_x1          ← 栈重排：把值留在栈顶以便 ireturn
   3: putfield      #1  // Field v:I
   6: ireturn         ← 返回写入的值
```

**非 int 专属**：root 用 javac 23 对 `long`/`double` 字段实测（[results/javap-write-accessor-shapes.txt](results/javap-write-accessor-shapes.txt)），同为返回值形：

```text
static long   access$002(W, long)    → dup2_x1; putfield lng:J; lreturn
static double access$102(W, double)  → dup2_x1; putfield dbl:D; dreturn
static int    access$202(W, int)     → dup_x1;  putfield in:I;  ireturn
```

即 `dup_x1`（单槽）/`dup2_x1`（双槽）+ `xreturn` 是 javac 对**所有**基本类型字段写访问器的统一形，descriptor 一律 `(LC;D)D`（返回被写字段的类型），**从不是 void `(LC;D)V`**。

## 一之二、决定性对照：**boolean 写访问器已恢复，int 写访问器被拒**（root 2026-10-04 追加，更正下文第二节的过度概括）

root 追加一个**同类内两种字段类型**的探针 [fixture/BA/BA.java](fixture/BA/BA.java)（真 javac 8 编译）：

```java
public class BA {
    private boolean flag = false;   // boolean 字段
    private int count = 0;          // int 字段
    class S { void setB(boolean b){ flag = b; } void setI(int i){ count = i; } }
}
```

两个写访问器的 **opcode 序列逐字相同**（root 以 awk 精确取单方法复核、`diff` 为空）：

| 访问器 | 描述符 | opcode 序列 | jarde 呈现 |
| --- | --- | --- | --- |
| `access$002` | `(LBA;Z)Z` | `aload_0 iload_1 dup_x1 putfield ireturn` | **恢复**：`arg0.flag = arg1; return arg1;` |
| `access$102` | `(LBA;I)I` | `aload_0 iload_1 dup_x1 putfield ireturn` | **拒绝** → 空 stub |

即**判别变量是字段描述符 `Z`，不是指令形**。root 读码定位判据：`build.rs:8848-8851` 的 `BooleanAccessorAssignment::prove` 要求 `field.owner == owner && field.descriptor == "Z" && field.access == Write && !field.is_static`，`descriptor != "Z"` 即 `return Ok(None)`。该处理器由**已合入的 `d09f5dea`**（"recover proved parent field writes and private setter helper"）交付，调用点 `build.rs:7293`，其文档（`build.rs:8751-8753`）自述"The **Java 8 private boolean setter helper**'s complete physical body. Its `dup_x1` has exactly three consumers: the receiver and value of one `putfield`, and the returned copy of that value."

**严重性实测**：int 访问器渲染为空 stub 但保留 `int` 返回类型 → 渲染源集 `javac --release 8` **exit 1**（"缺少返回语句"）→ **整类不可编译**（响亮失败，非静默偏离）。见 [results2/BA-javac-errors.txt](results2/BA-javac-errors.txt)、[results2/BA-rendered.txt](results2/BA-rendered.txt)。

> **root 自查纠错（本文件第二、五节的表述被本对照收窄）**：下文第二节写"`accessor.rs:480` 的 `if returns.is_some()` **有意拒绝**该形"、第五节写"写访问器臂**从未被任何真实 javac 产物行使过**"——**两处都不完整**。(1) `accessor.rs:480` 的拒绝是**访问器折叠规则**（`accessor@1`）的路径，而 `d09f5dea` 在 `build.rs` 另建了 `BooleanAccessorAssignment` 通路，**boolean 写访问器经该通路已恢复**（上表实证），故"一律拒绝"不成立。(2) "从未被真实产物行使"仅对 **`tests/fixtures` 语料**成立（root 普查：532 类中含 `access$` 的为 0、`p3_accessor_edges.rs:146-149` 合成访问器全为读形），但 `d09f5dea` 自己的 fixture（`openspec/evidence/…/dt29-reference-cast-audit/fixtures/private-field/`）确实行使了 boolean 写形——root 先前只普查了 `tests/fixtures`，**范围过窄导致结论过强**（正是 handoff「空查询不能证明不存在」纪律的情形）。
>
> **更正后的准确表述**：写访问器的**指令形已被支持**（`dup_x1` 值返回形有专门处理器），缺口是**该处理器把字段类型限定为 `boolean`**，故 `int`/`long`/`double`/引用类型字段的写访问器仍被拒 → 空 stub → 整类不可编译。这把缺口从"新机制"**收窄为"既有处理器的类型泛化"**，颗粒度显著下调（原估"中"，现判**窄到中**：把 `field.descriptor != "Z"` 放宽为"描述符与访问器返回类型一致且属单槽/双槽可 `dup_x1`/`dup2_x1` 表达的形"，并为每种类型补正例；`long`/`double` 用 `dup2_x1` + `l/dreturn`，root 已在第一节 javap 实录）。
>
> **另有一次假零事故（诚实登记）**：root 首次跑本探针时 `cd /tmp/boolacc` 后用**相对**路径调 `target/debug/jarde-cli`，而该二进制此前已被 `rm -rf target` 清掉，得 exit 127、输出文件只有 104 字节错误信息；对其计 `@bytecode` 得 **0**，一度被读成"两个访问器都恢复了"。这正是 handoff「假零结果」纪律描述的情形，且是 root **在把该纪律写进 handoff 之后**又犯的一次。改用绝对路径 + 重建二进制 + 检查文件含 jarde 自述头后，才得到上表（quotes=3、int 形拒绝）。

## 二、拒绝是有意的，且渲染为空 stub（响亮，非静默）

> **本节及第五节的"一律拒绝/从未被行使"表述已被第一节之二收窄**（boolean 写访问器经 `d09f5dea` 的 `BooleanAccessorAssignment` 通路**已恢复**，缺口实为该处理器的**字段类型限定为 `Z`**）。下文保留原始记录不改写，阅读时以第一节之二为准。

`accessor.rs:474-489`（`FieldAccess::Write` 分支）：

```rust
if returns.is_some() {
    return Err(Refusal::shape("jre_accessor_field", format!(
        "the body of `{}{}` writes the field `{name}` and returns a value, \
         and the call site that used to spell it is a statement", …)));
}
```

即写访问器**只要返回一个值就拒绝**。渲染结果是空 stub（[results/WA-javac23-family-rendered.txt](results/WA-javac23-family-rendered.txt)）：

```java
static int access$002(WA arg0, int arg1) {
    // jarde: not recovered: the recovery run for `access$002(LWA;I)I` produced no statement
}
```

调用点保留物理调用（`WA.access$002(WA.this, arg1);`），未折叠为 `this.v = arg1;`。空 stub + `int` 返回 → `javac` 报"缺少返回语句"→ 整类不可编译。**这是响亮失败**（不产生"可编译但行为不同"的文本），符合核心不变量；但它是真实 Java 8 覆盖缺口。

## 三、与 DT-03 `getClass` 缺口**独立**（关键：已排除混淆）

root 一度怀疑写访问器缺口是 DT-03 折叠失败的**后果**（折叠不成功 → 访问器不被消隐）。用 **javac 23** 编译 `WA.java`（null-check 为 `requireNonNull`，DT-03 折叠**可用**）排除该混淆：

| 编译 | DT-03 折叠 | 写访问器 `access$002` 体 |
| --- | --- | --- |
| javac 23（`requireNonNull`） | **成功**（`o.new Setter().set(5)` 折叠、`access$` 从 `Setter` 源码区消隐） | **仍空 stub、仍 not recovered** |

即 DT-03 折叠成功时写访问器**依旧**不恢复——两缺口正交。DT-03 缺口是"限定分配点的 null-check 拼写"（在 `init.rs`/`member_inner.rs`），本缺口是"访问器体的返回值形写"（在 `accessor.rs:480`），**落点、判据、所有者都不同**。

> **root 自查纠错（诚实登记）**：本节结论曾险些写错。root 初次用 `--class 'WA$Setter'`（子类）渲染，未见折叠，误以为"javac 23 下折叠也失败"——那是**调用姿势错误**（家族折叠须 `--class WA` 渲染根类，子类单独渲染不触发折叠）。改用 `--class WA` 后确认 javac 23 折叠成功、写访问器仍失败，才得到上表的正交结论。这与 [DT-03 巡查](../qualified-outer-alloc-getclass-patrol/README.md) 中 root 用错 `--class` 是同一类脚手架错误，已固化进 handoff。

## 四、语料盲区：全仓 0 个真实 `access$` 产物

root 全仓普查（`git ls-files '*.class'`，含 `tests/fixtures` 与全部 `openspec/evidence`）：**含 `access$` 方法的 class 文件 = 0**。且 `p3_accessor_edges.rs:146-149` 合成的访问器**全部是读形**（`getfield; ireturn`）。

即 **accessor 规则（P3 2.2 / `accessor@1` / 验收 A12）从未被任何真实 javac 产物或合成写形测试行使过**。这与 DT-03 缺口是**同一个失败模式**——"验收锚恰好覆盖不到真实产物的变化维度"——但盲区更大：DT-03 至少有 8 个真实 `requireNonNull` fixture（只是缺 `getClass` 形），而 accessor 写形**连一个真实产物 fixture 都没有**。

## 五、归属与处置

**归属**：EM-15「合成 getter/桥的安全内联」（[expressions-misc.md](../../jadx-feature-inventory-2026-09-27/expressions-misc.md)，summary 记为"EM-15 是合成访问器内联机制"）；实现落点 `crates/jarde-java/src/accessor.rs`（`FieldAccess::Write` 分支）。相关已合入片 `d09f5dea`（"recover proved parent field writes and private setter helper"，改 `build.rs`+250/`field.rs`+40）处理的是**父类字段写**（`B extends A` 写 `A.visible`），其 design 明确 Non-Goal："不宣称所有 JDK 编译器版本或 private accessor 生成方式兼容"、"private accessor 常使用栈重排表达赋值结果；本项只接受已完整证明的固定形态，不能把任意 `dup*` 都解释成赋值"——即 `d09f5dea` **已知**写访问器用 `dup` 栈重排但**有意只接受它证明的固定形**，本缺口（内部类写外部私有字段的返回值形）不在其范围。

**处置：应独立立项**（不并入 DT-03 片）。MVP 范围建议（须先取证再钉死，root 未做完整设计）：

1. **先取证**：构造真 javac 8 + javac 23 双腿的写访问器 fixture（`int`/`long`/`double`/引用类型各一），确认 `accessor.rs` 的读形判据（`FieldAccess::Read`）是否也只在合成测试中行使过、真实读形产物是否存在。
2. 判据扩展方向：`FieldAccess::Write` 分支接受"返回值 == 被写字段类型"的返回值形（`dup_x1`/`dup2_x1` + `putfield` + `xreturn`），把"返回值被调用点丢弃"作为**调用点**事实核对（与 DT-03 片的 `pop` 判据同构），而非在访问器体内拒绝返回值。
3. **健全性负例**：`accessor.rs:17` 明文"an accessor that computes, casts, calls or writes twice is refused"——`I1.access$012`（复合 `counter += n`，含 `iadd` 计算）属"computes"形，**必须保持拒绝**，不得因放宽返回值形而误纳。这是本缺口最可能引入的过度放宽，须冻结为负例。
4. 验收须**双腿** + 真实产物 fixture（填补第四节的 0 盲区），不得只用合成读形。

**优先级**：低于 DT-03 片（后者已派发）。DT-03 是"目标层级 Java 8 上常见形 `outer.new Inner()` 整方法被拒"；本缺口是"内部类写外部私有字段整类不可编译"，同样常见，但落点 `accessor.rs` 与 DT-03 的 `init.rs`/`member_inner.rs` 不冲突，可串行处理。

原 class 为行为基准：`WA` → `5`（[fixture/WA.java](fixture/WA.java) `main` 的 `System.out.println(o.v)`）；`I1` → `3`/`11`/`16/16`/`42`/`7`/`9/9`（六行，见 [DT-03 巡查](../qualified-outer-alloc-getclass-patrol/README.md) 的 `I1` 基线）。
