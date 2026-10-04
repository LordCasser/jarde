## ADDED Requirements

### Requirement: 分配限定符的成员构造在真 javac 8 的 null 检查下 SHALL 恢复

系统 SHALL 把分配限定符形 `new Outer().new Inner(…)` 中被**丢弃**的 null 检查（`dup; <discarded null check>; pop`，两种拼写之一：javac 9+ 的 `Objects.requireNonNull` 或真 javac 8 的 `Object.getClass`）识别为**消费该外层实例的成员构造站点自身的一部分**，而不是外层分配站点的一个游离读者；从而真 javac 8 产物（其对新分配外层实例无条件插入该检查）与 javac 9+ 产物（其不插入）**都**恢复为源级限定语法 `new Outer().new Inner(…)`。

识别 null 检查 SHALL 复用既有谓词 `facts.rs::is_discarded_null_check`（`recover-javac8-getclass-null-check-idiom` 交付），**不得**新建第二套拼写判据。系统 SHALL **不得**引入 `java_release`/`major_version` 或任何按产出编译器版本分支的判据（两种产物 major version 均为 52，且违反 `classfile.rs` 的"never over the compiler that produced it"原则）。

**核心不变量 SHALL 保持**：外层实例被**多处真实消费**（非被丢弃的 null 检查）时仍 SHALL 拒绝——"一个构造实例只有一个 Java 拼写位"（`single_use_at`、读者门 `readers.len() != 1`）的语义不得因本能力放宽。只有"被 `is_discarded_null_check` 证明、且返回值被紧随的 `pop` 丢弃"的三元组才归入站点自有。用户显式写的 `o.getClass();` 语句（其值不被丢弃，或不在分配序列内紧邻构造器）SHALL 保留为一条语句，不得被误折叠。

#### Scenario: 真 javac 8 的分配限定符形可恢复

- **WHEN** 以真 javac 8（Corretto 1.8.0_432，无 `--release`）编译的 `N1` 族（`N1.main` 含 `new N1().new Inner(3).total()`，其字节码在嵌套分配 `invokespecial N1.<init>` 之后、成员构造器之前有 `dup; invokevirtual Object.getClass:()Class; pop`）经 `class-source` 呈现
- **THEN** `new N1().new Inner(3)` 折叠为源级限定语法，`N1` 族源码区引注数从 13 降为 0（或与 javac 9+ 腿同形）；渲染源集经 `javac --release 8` 编译 exit 0，`java -Xverify:all` 运行输出 `10`/`7`/`13` 与原 class 逐行一致

#### Scenario: javac 9+ 产物零回退

- **WHEN** 同一 `N1.java` 以 javac 23 `--release 8` 编译（分配限定符**不**插入任何 null 检查），以及 `recover-javac8-getclass-null-check-idiom` 交付的 8 个 `requireNonNull` fixture 与参数限定符形（`N1x`/`Wrap`）经呈现
- **THEN** 渲染文本与修改前**逐字节相同**——本片只动分配限定符路径，参数限定符形与隐式 this 形不受影响

#### Scenario: 带实参与无实参的分配限定符形都被修

- **WHEN** 分配限定符形分别带实参（`new Outer().new Inner(3)`）与不带实参（`new Outer().new Inner()`）
- **THEN** 两形都恢复——不得只修锚覆盖的带实参形（验收锚不得是唯一正例）

#### Scenario: 外层实例被多处真实消费时仍拒绝

- **WHEN** 外层实例被非 null-check 的多处读取消费（如 `Outer o = new Outer(); o.f(); o.new In();`，或 null 检查的返回值未被 `pop` 丢弃而是被后续读取）
- **THEN** 保持既有拒绝（`single_use_at` / 读者门的核心不变量不破）——本能力只放宽"被丢弃的 null 检查三元组"，不放宽"实例有多个真实读者"

#### Scenario: 用户显式 getClass 语句不被误折叠

- **WHEN** 方法体先有一条用户显式写的 `o.getClass();`（返回值同样被丢弃，但位于 `new` **之前**、不在分配序列内），随后才是 `o.new In()`
- **THEN** 用户的 `o.getClass();` SHALL 保留为一条语句；只有分配序列内紧邻构造器的那一条 null 检查被识别为站点自有

#### Scenario: 判据不得按编译器版本分支

- **WHEN** 审查本片的生产 diff
- **THEN** 其中 SHALL 不出现任何 `java_release`、`major_version` 或等价的按版本分支判据；恢复与否只取决于"是否存在被丢弃的 null 检查"这一字节码事实
