## ADDED Requirements

### Requirement: 限定外部实例构造的 null 检查判据 SHALL 识别两种编译器拼写

系统 SHALL 把源级限定表达式 `outer.new Inner(…)` 的 null 检查判据建立在**该检查的事实**上——分配序列内一条**返回值被丢弃**的调用——而不是建立在某一个编译器对该事实的**拼写**上。具体地，`init.rs` 的成员构造证明与 `member_inner.rs` 的成员调用路径 SHALL 各自把自己读取的表示投影为同一组事实（调用种类、符号 owner、符号 name、符号 descriptor、是否接口引用），并调用**同一个**谓词判定，该谓词 SHALL 成对接受且仅接受两种拼写：

- `invokestatic java/util/Objects.requireNonNull:(Ljava/lang/Object;)Ljava/lang/Object;`（javac 9+）
- `invokevirtual java/lang/Object.getClass:()Ljava/lang/Class;`（真 javac 8）

调用种类与符号 SHALL **成对匹配**，其叉积（`invokestatic Object.getClass`、`invokevirtual Objects.requireNonNull`）SHALL 被拒绝，因为那不是任何 javac 的产物，接受它会把允许集放宽到本判据没有证据支撑的形。

系统 SHALL **不得**引入 `java_release`、`major_version` 或任何按产出编译器版本分支的判据：两种拼写的产物 class 文件 major version 均为 52，版本字段无法区分；且 `classfile.rs` 已固化"答案 stated over the facts that class really carries — never over the compiler that produced it"。

既有的**位置约束 SHALL 逐字保留**，不因增加拼写而放宽：`[qualifier, copy, check, pop]` 的连续性、`check` 锚定在分配点、`pop` 的 opcode `0x57`、以及成员调用路径的实参区间检查（实参 BCI 须落在 `pop` 之后、构造器调用之前）。"返回值被 `pop` 丢弃"是**调用点**的事实，SHALL 保留在两处调用点各自检查，不得移入拼写谓词。

当两种拼写都不成立时，系统 SHALL 保持既有拒绝语义（响亮失败、整方法不投影），且拒绝文本 SHALL 不再指称单一拼写，以免文本与判据不符。

#### Scenario: 真 javac 8 产物的限定外部实例构造可恢复

> **root 验收更正（2026-10-04，实现者按停手条件 (a) 报告、root 独立复核后确认本 Scenario 的前提错误）**：本 Scenario 原写"`N1.main` 含分配限定符 `new N1().new Inner(3).total()`；**两处的 null 检查均为** `dup; aload_1; dup; invokevirtual Object.getClass:()Class; pop`"，并据此要求"两处限定构造都折叠、整族引注数从 18 降为 0"。**前半句错**：`getClass()` 确实在真 javac 8 的两形都出现，但 **javac 23 对分配限定符形完全不发射任何检查**（冻结 `N1.main` javap 实录：`20: invokespecial N1.<init>` 之后直接 `23: iconst_3; 24: invokespecial N1$Inner.<init>`）。故 `init.rs` 的分配限定符臂本就按"无检查"形编写，真 javac 8 在同位置插入的 `dup(23); invokevirtual getClass(24); pop(27)` 使**嵌套分配 BCI 16 的实例只被"本 build 引注的指令"读取**，故在 **`new@1` 的嵌套构造站点层**被拒——实测拒绝码为 `jre_new_interleaved_effect` + `jre_new_shape`（`init.rs:412` 的 shape 判据与 interleaved-effect 判据），诊断原文："the construction at BCI 16 was not presented: the instance the allocation at BCI 16 builds is read only by instructions this build quotes (BCIs 23), so the construction has no place in the body"、"4 construction candidate(s) read under new@1: 2 presented as `new`, 2 refused"。**与拼写判据无关**。因此本片的两处拼写判据修复**在结构上无法触达分配限定符形**，本 Scenario 的 THEN 不可达成。
>
> **本 Scenario 的实际达成范围（root 合并后独立实测）**：**参数限定符形与隐式 this 形已修复**——`N1x`/`Wrap` 族（真 javac 8 产物）源码区 quotes=0，呈现 `return arg1.new Inner(9).total();` 与 `return new Inner(arg1);`，渲染源 `javac --release 8` exit 0，`java -Xverify:all` 输出与原 class 逐行一致（`7`/`13`/`10`）。**分配限定符形仍未折叠**：以真 javac 8 重编冻结 `N1.java`，源码区 quotes=13、not-recovered=1，两形均未折叠。该残留属**机制扩展**（须让分配限定符臂接受实参窗口内被丢弃的检查三元组），**另立后续片**，不在本片范围。root 的 spec 取证不足是本次未达成的原因，非实现偏差；实现者未自行扩权，处置正确。
>
> （口径说明：原文的"18 引注"是全文件 `grep -c '@bytecode'`，含 JSON 报告尾；源码区口径为 13。后续一律以源码区口径为准。）

- **WHEN** 以真 javac 8（Corretto 1.8.0_432，无 `--release`）编译的 `N1` 族（`N1$Stat.use` 含参数限定符 `outer.new Inner(9).total()`，`N1.main` 含分配限定符 `new N1().new Inner(3).total()`；~~两处的 null 检查均为 `dup; aload_1; dup; invokevirtual Object.getClass:()Class; pop`~~ ← **root 更正：仅参数限定符形如此，分配限定符形在 javac 23 下无任何检查**）经 `class-source` 呈现
- **THEN** ~~两处限定构造都折叠为源级 `outer.new Inner(…)` / `new N1().new Inner(…)` 语法，整族引注数从 18 降为 0（或与 javac 9+ 腿同形）~~ ← **未达成，见上更正**；**实际达成**：参数限定符形折叠为 `outer.new Inner(…)`、渲染源集经 `javac --release 8` 编译 exit 0、`java -Xverify:all` 运行输出与原 class 逐行一致

#### Scenario: javac 9+ 产物零回退

- **WHEN** 同一份 `N1.java` 以 javac 23 `--release 8` 编译（null 检查为 `invokestatic Objects.requireNonNull`），以及既有 8 个携带该拼写的 fixture 类（`p3-lambda-adaptation/v8/BoundNullLambdaAdaptationProbe`、`proved-java-structure/anonymous-member-base/{AnonymousMemberBase,$1,$2}`、`recover-generic-enclosing-member-call-sites/matrix/{UseGenericObject,UseGenericTyped,UsePlain,UsePlainRaw}`）经呈现
- **THEN** 渲染文本与修改前**逐字节相同**；`tests/inner_class_static_mixed_folding.rs` 的两条文本断言（`new N1().new Inner(3).total()`、`return arg1.new Inner(9).total();`）与 `recover-proved-member-inner-construction` 全部测试继续通过

#### Scenario: 用户显式 `getClass()` 语句不得被误折叠

- **WHEN** 方法体先有一条用户显式写的 `o.getClass();`（其返回值同样被 `pop` 丢弃，位于 `new` **之前**），随后才是 `o.new In()`（javac 插入的 null 检查位于分配序列**内**、紧邻 `invokespecial`）
- **THEN** 呈现中用户的 `o.getClass();` SHALL 保留为一条语句，不得被吞掉或当作 null 检查折叠；只有分配序列内的那一条被识别为 null 检查；整类可编译且行为与原 class 一致

#### Scenario: 位置约束不成立时仍拒绝

- **WHEN** 该调用后缺少 `pop`（返回值未被丢弃）、或 `[qualifier, copy, check, pop]` 四条不连续、或成员调用的实参 BCI 不落在 `pop` 之后与构造器调用之前
- **THEN** 按既有拒绝码响亮失败，不因"拼写匹配"而放行——增加拼写不改变位置约束

#### Scenario: 拼写的叉积不被接受

- **WHEN** 一条调用是 `invokestatic java/lang/Object.getClass:()Ljava/lang/Class;` 或 `invokevirtual java/util/Objects.requireNonNull:(Ljava/lang/Object;)Ljava/lang/Object;`（种类与符号来自不同拼写的组合）
- **THEN** 拒绝，不得当作 null 检查——本判据只对两种成对拼写有证据

#### Scenario: 判据不得按编译器版本分支

- **WHEN** 审查本片的生产 diff
- **THEN** 其中 SHALL 不出现任何 `java_release`、`major_version` 或等价的按版本分支判据；两种拼写的接受与否只取决于调用自身的事实
