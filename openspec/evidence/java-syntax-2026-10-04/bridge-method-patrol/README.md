# 桥方法呈现巡查（2026-10-04）——硬编译错误（name clash）

泛型/协变域巡查（主线 `30e54613`；**巡查复用 spn worktree 的 HEAD 版 jarde-cli——已核实二进制新于源码且 crates/src/tests 无改动，忠实反映 HEAD**）。固定转录 [fixture](fixture/)（BR 家族五类；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 orig.out（`Base`/`s`/`0`）。

## 表现：javac 合成的桥方法被显式呈现 → 源码非法

javap 确认字节码中确有 `ACC_BRIDGE|ACC_SYNTHETIC` 成员（`BR$Base.next()LBR$Node;`、`BR$StrBox.get()Ljava/lang/Object;`+`set(Ljava/lang/Object;)V`、`BR$Impl.compareTo(Ljava/lang/Object;)I`）。Jarde 把它们与真实成员**并列呈现**：

```java
public BR$Base next() { return new BR$Base(); }   // 真实协变覆写
public BR$Node next() { return this.next(); }     // 桥——源码层禁止声明
```

```java
class BR$Impl implements Comparable {
    public int compareTo(BR$Impl arg1) { return 0; }
    public int compareTo(java.lang.Object arg1) { return this.compareTo((BR$Impl) arg1); }  // 桥
}
```

**javac 拒编实测**（[results/](results/) 内的 StrBox.java / BR$Base.java）：

- `BR$Base`：`错误: 已在类 BR$Base中定义了方法 next()`（两同名同参方法仅返回类型不同）
- `BR$StrBox`：`错误: 已在类 BR$StrBox中定义了方法 get()`

即**含协变返回覆写的类不可重编**——真实代码高频形态（泛型容器子类、Builder、返回自身类型的接口实现）。

### 2026-10-04 root 事实修正（实现者复核发现，已实测确认）

本巡查把 `BR$Impl` 一并归入"javac 拒编"**是错的**，此处更正并给出精确判据边界：

| 桥形态 | 源码并列声明 | javac | 性质 |
| --- | --- | --- | --- |
| **协变返回擦除**（`Base next()` vs `Node next()`；`String get()` vs `Object get()`） | 同名**同参**、仅返回类型不同 | **错误：已在类中定义了方法 X()** | **硬编译错误**（源码层禁止） |
| **参数 cast 擦除**（`compareTo(Impl)` vs `compareTo(Object)`；`set(String)` vs `set(Object)`） | 同名**不同参** → 合法重载 | exit 0 | 可编译；桥可见只是**形态冗余**（源码不会写它） |

实测（root，2026-10-04）：`class Impl implements Comparable { public int compareTo(Impl o) {…} public int compareTo(Object o) {…} }` → `javac --release 8` **exit 0**；`class Both extends Box { String get(); void set(String); void set(Object) }`（仅参数桥，无协变 get 桥）→ **exit 0**；而 `String get()` + `Object get()` → **报错**。

**故 `BR$Impl` 的基线本就整类可编译**，其"桥可见"是形态问题而非编译问题。据此修正两门的性质与优先级：

- **门 1（擦除返回子类型）= 硬编译错误修复**，必需，优先级最高。
- **门 2（参数 cast 形）= 合成成员消隐**（形态改善），**非可编译性必需**；且按消隐前置不变量（见 [header-invariant/](header-invariant/)），泛型接口在类头拼为裸类型时**必须保持桥可见**，否则 javac 报 missing-override——即门 2 的投影以类头类型实参投影为前置，该前置由 [recover-parameterized-interface-headers](../../../changes/recover-parameterized-interface-headers/) 供给。

影响面重述（更准确）：真实代码中**协变返回覆写**才是"整类不可重编"的成因；泛型特化（`Comparable<T>` 实现、`Box<String>` 子类）在桥可见时仍可编译，其代价是源码形态含合成成员声明。

## 根因

`crates/jarde-java/src/bridge.rs` 的 `bridge@1` 规则目标是"把桥的已证转发呈现为它所转发的调用"（drop 已证 erasure cast），**产出仍是一个成员声明**；规则从未隐藏桥成员本身。对**协变返回擦除桥**，Java 源码不允许该声明（同名同参、仅返回类型不同是非法重载），故呈现即硬错误；对**参数 cast 擦除桥**，声明合法但源码从不书写它（javac 会为泛型特化重新生成），故呈现是形态冗余而非编译错误（见上节事实修正）。判据本身健全（`MethodFacts::is_bridge` 读 ACC_BRIDGE 声明事实、非由体形推断），缺的是**成员级消隐**——且消隐须以"javac 能重建该桥"为前置（协变返回桥的重建来自父类型契约，参数 cast 桥的重建来自类头类型实参投影）。

## 同族先例

与已闭合的 `access$000` 消隐（instance-folding 片的 `HiddenAccessBridge`）、lambda 伴生消隐（`recover-lambda-inline-bodies`）完全同族：**javac 合成成员按声明事实隐藏**。本片是同一原则的第三个成员族。

## 处置方向

`recover-bridge-member-suppression`（窄切片）：类源装配时隐藏 `ACC_BRIDGE` 成员（复用既有消隐语境机械——access$000 的 `HiddenAccessBridge` derived 锚模式），不呈现其声明与体；真实覆写成员不变。判据仅 ACC_BRIDGE 声明事实（不新增机制、不由体形推断——与 bridge.rs 既有原则一致）。BR 家族三类恢复且可重编、行为一致；`bridge@1` 规则的既有转发呈现测试（用于 access$/bridge **调用位**）零回退。

原 class 为行为基准。
