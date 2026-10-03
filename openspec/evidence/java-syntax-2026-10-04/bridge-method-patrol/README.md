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

即**任何含协变返回覆写或泛型参数特化的类都不可重编**——真实代码高频形态（`Comparable<T>` 实现、Builder、泛型容器子类）。

## 根因

`crates/jarde-java/src/bridge.rs` 的 `bridge@1` 规则目标是"把桥的已证转发呈现为它所转发的调用"（drop 已证 erasure cast），**产出仍是一个成员声明**；规则从未隐藏桥成员本身。而 Java 源码不允许声明桥（仅返回类型不同的重载非法），故呈现即硬错误。判据本身健全（`MethodFacts::is_bridge` 读 ACC_BRIDGE 声明事实、非由体形推断），缺的是**成员级消隐**。

## 同族先例

与已闭合的 `access$000` 消隐（instance-folding 片的 `HiddenAccessBridge`）、lambda 伴生消隐（`recover-lambda-inline-bodies`）完全同族：**javac 合成成员按声明事实隐藏**。本片是同一原则的第三个成员族。

## 处置方向

`recover-bridge-member-suppression`（窄切片）：类源装配时隐藏 `ACC_BRIDGE` 成员（复用既有消隐语境机械——access$000 的 `HiddenAccessBridge` derived 锚模式），不呈现其声明与体；真实覆写成员不变。判据仅 ACC_BRIDGE 声明事实（不新增机制、不由体形推断——与 bridge.rs 既有原则一致）。BR 家族三类恢复且可重编、行为一致；`bridge@1` 规则的既有转发呈现测试（用于 access$/bridge **调用位**）零回退。

原 class 为行为基准。
