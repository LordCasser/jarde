# `recover-assert-statement-sugar` 实现证据（`-asg`）

实现取证的固定转录。行为基线与 A1 固定 fixture 沿用本目录上一层（`../fixture/`、`../results/A1.txt`）；
Java 源与 `AssertProbe` 原类来自 2026-09-23 审计（SHA 与其记录一致）。

## 1.1 取证：javac 合成物精确形状（javap，javac 23 `--release 8`）

**字段**（`A1` 与 `A1$Sub` 各自持有一份，二进制彼此独立）：

```
static final boolean $assertionsDisabled;
  descriptor: Z
  flags: (0x1018) ACC_STATIC, ACC_FINAL, ACC_SYNTHETIC
```

**`<clinit>` 单行**（两形共用，`X` 见下）：`ldc X; invokevirtual Class.desiredAssertionStatus()Z;
ifne 12; iconst_1; goto 13; 12: iconst_0; 13: putstatic $assertionsDisabled`。

**X 规则（实测，两级嵌套逐一 javap）**：X 恒为**最外层外围类**的 Class 字面量——`p.Outer$Mid` 与
`p.Outer$Mid$Deep` 的 `<clinit>` 都 `ldc class p/Outer`；A1 家族里 `A1$Sub` 也读 `A1.class`（本文件
顶层 fixture 复核一致）。每级 class 文件自己的 `InnerClasses` 属性同时列出全部外围链行，最外层可从
本类文件独立判定。`assert true` 被折叠为零 lowering（无字段、无 `<clinit>`），不进入本模式。

**使用点守卫两形**（`check` 带消息 / `plainAssert` 无消息）：

```
0: getstatic $assertionsDisabled:Z
3: ifne  END          ← 守卫出口（断言关闭时整体跳过）
…求值 cond…
N: if<cc> END         ← cond 为假跳到抛出（比较形直接用反极性指令）
new AssertionError; dup; [求值 msg;] invokespecial <init>(…); athrow
```

msg 按构造器形选重载：`"s"+x` → `(Ljava/lang/Object;)V`（jarde 呈现为 `(java.lang.Object)` 位置转换）、
`int v` → `(I)V`、无 msg → `()V`。msg 在 `new` 之后、`<init>` 之前求值（抛出前、仅失败路径）。

**呈现位与隐藏挂点（阅读落点）**：守卫由方法体恢复的普通 `If`/`Throw`/`New` AST 呈现
（`crates/jarde-java/src/build.rs`→`emit.rs`）；`<clinit>` 行是 `FieldAssign`（blank static final 无
receiver）；字段声明在 `src/class_source.rs` 装配的字段循环；类文本首次装配在
`src/facade.rs::prepare_physical_class_source` 末端 `source_text`，其语境过滤先例即
`DeclarationForm::StaticInitializer` 驱动的 return 抑制（emit.rs `body`/`stmts`）。跨成员模式识别挂
在同函数：每成员保留 AST（`ClassSourceMethodAst`）+ 每成员 Fieldref 操作
（`ClassSourceEnumSwitchFieldUse`，物理 census 通道，不受 evidence 选择影响——`RecoveryReport.fields`
受 RuleDetails 选择门控，不能用作 census）。

## 1.2 变体与负例（固定，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）

| fixture | 形态 | 两态行为（`-Xverify:all`，实现前后逐字一致） | 实现后呈现 |
| --- | --- | --- | --- |
| `AssertProbe`（正） | msg 为带副作用方法调用 `detail(value)`，cond 为 `guard(value)` | 默认 `0|0;0|0`；`-ea` `1|0;bad:-1|2|1` | `assert guard(arg0) : detail(arg0);`，合成物消隐；重编两态一致 |
| A1 家族（正，锚定） | 两形 + 嵌套类 `Sub` 独立开关 | 默认 `10/25/2/-2`；`-ea` `10/25/2`+AssertionError | `assert`×3，字段/clinit×2/守卫全消隐；重编两态一致 |
| `AssertProbe-extra`（负） | 守卫体内额外语句：BCI 6 插入 `iinc 0,0`（`value += 0`，手工字节码） | 同 AssertProbe 原类 | 守卫 + 字段 + clinit 行逐字保持（`value = value + 0;` 在守卫内） |
| `AssertProbe-wrong-owner`（负） | `<clinit>` 仅改一个 `ldc` 常量池索引 → 读 `StringBuilder.class` | 默认 `0|0;0|0`；`-ea:AssertProbe -da:java.lang.StringBuilder` 时 `0|0;0|0`（与原不同，见 09-23 审计） | 逐字保持（clinit 行如实呈现 `java.lang.StringBuilder.class…`，不折叠） |
| `MixedShapes`（负） | 混合形态：`assert a \|\| b` 降为嵌套双测、`assert !f : counter` 的 msg 未恢复、`assert a && b` 成员整体 fallback | 正常运行 | **全类逐字保持**（all-or-nothing：任一使用点不可折叠则字段保留、全部守卫保持） |

负例逐字保持的判定：`git stash` 基线构建与实现构建对同一 class 双腿转录，`diff` 为空
（`MixedShapes` 实测 `V1-BYTE-IDENTICAL`）；其余负例断言三件合成物仍在 + 无 `assert` 语句。

## 2.1 模式判据落点（实现）

- **判据**（`crates/jarde-java/src/asserts.rs`，全部结构匹配已恢复 AST，非重新译码）：
  字段 `$assertionsDisabled` ∧ `ACC_SYNTHETIC|ACC_STATIC|ACC_FINAL` ∧ 描述符 `Z`；`<clinit>` 恰一行
  `(!X.class.desiredAssertionStatus() ? 1 : 0) % 2 != 0`（X == 由本类 `InnerClasses` 链推得的最外层类；
  wrong-owner 在此被拒）；使用点恰为 `if (!Path.$assertionsDisabled) { if (!cond') throw new AssertionError(msg?); }`
  无 else、无旁句；`cond'` 取反（`Not(e)`→e、比较反极性、其余布尔包 `!`）；msg 恰 0/1 个参数，
  `(Object)` 位置转换剥除。
- **消隐语境**：按类 all-or-nothing——census（每成员 Fieldref 操作）要求该字段的每个物理读恰为被折叠
  守卫的 `getstatic`、唯一写在 clinit 行；否则全类保持。折叠后：字段声明隐藏
  （`ClassSourceProjectionInputs.hidden_fields` 新通道 + `HiddenAssertSwitchField` derived 锚）、
  clinit 行移除、剩余为空则整个 `<clinit>` 成员省略（`omitted_methods`）。守卫成员文本经
  `assert_projection_text`（保留原 envelope）替换，并同时经 `member_texts`+emission 通道暂存（供
  静态成员折叠路径重拼写）。
- **语境互锁（保守拒绝）**：接口/注解/枚举/module；initializer 组或 enum 组已投影；已有 lambda/
  array 成员文本暂存或 helper 省略；嵌套 enum/annotation/匿名接口候选存在；整数常量名候选存在；
  实例折叠 joint 计划遇到已消隐类时拒绝（其成员重跑会重新生成守卫文本）。
  以上全部保持实现前呈现。

## 3.2 三方对照与 corpus

见 [results/README.md](results/README.md)（原类两态 / 固定 JADX(dev) / Jarde 重编逐路径 + 输出 SHA、
corpus 双腿扫描）。
