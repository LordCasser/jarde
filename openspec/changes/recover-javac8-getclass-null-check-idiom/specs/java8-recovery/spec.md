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

- **WHEN** 以真 javac 8（Corretto 1.8.0_432，无 `--release`）编译的 `N1` 族（`N1$Stat.use` 含参数限定符 `outer.new Inner(9).total()`，`N1.main` 含分配限定符 `new N1().new Inner(3).total()`；两处的 null 检查均为 `dup; aload_1; dup; invokevirtual Object.getClass:()Class; pop`）经 `class-source` 呈现
- **THEN** 两处限定构造都折叠为源级 `outer.new Inner(…)` / `new N1().new Inner(…)` 语法，整族引注数从 18 降为 0（或与 javac 9+ 腿同形）；渲染源集经 `javac --release 8` 编译 exit 0，`java -Xverify:all` 运行输出 `10`/`7`/`13` 与原 class 逐行一致

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
