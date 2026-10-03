# 残余边界的决定性实证：family 折叠失败 + 结构反射 = ncl 引入的静默偏离（2026-10-04 root）

`recover-nested-class-literal-values` 修正轮的实现者诚实单列了一个"残余边界"并请求 root 机制裁决。root **不凭描述裁决**，构造了命中该边界的 fixture（RF）并实测判定——结论：**这是验收阻塞点**，因为它是 ncl 准入放宽**引入的净倒退**，且形态恰是核心不变量禁止的"可编译且行为不同"。

## RF fixture（[fixture/RF.java](fixture/RF.java)）

```java
public class RF {
    static class Inner { static final int K = init(); static int init() { return 7; } }  // <clinit> 使折叠被拒
    public static String simpleName() { return Inner.class.getSimpleName(); }             // 结构反射消费池形字面量
    public static void main(String[] a) { System.out.println(simpleName()); System.out.println(Inner.K); }
}
```

- 原类 `java -Xverify:all`：`Inner` / `7`（[results/original.out](results/original.out)）。
- javap 实证：`RF$Inner` 有 `<clinit>`（`static final int K = init()`）→ 触发 facade.rs:20372 "member fold member has an enum or initializer projection this fold does not carry" → 折叠被拒。
- `Inner` 是 `RF` 的**直属成员**（深度 1），且被 `getSimpleName()`（结构反射）消费——正是残余边界的三要素（family 口径 ∧ 直属成员 ∧ 折叠失败 ∧ 结构反射）。

## 决定性对照（[results/](results/)）

| 步骤 | 结果 | 文件 |
| --- | --- | --- |
| jarde family 口径呈现 `RF` | `simpleName()` → `return RF$Inner.class.getSimpleName();`，**0 引注**（自称完整恢复） | [results/RF-jar-fold-refused.txt](results/RF-jar-fold-refused.txt) |
| 分离单元 `RF$Inner.java` 声明 | `class RF$Inner extends java.lang.Object`（池形名，顶层类） | — |
| 家族一起 `javac --release 8` | **exit 0**（可编译） | — |
| 重编家族 `java -Xverify:all` 运行 | **`RF$Inner` / `7`** | [results/recompiled-flat-family.out](results/recompiled-flat-family.out) |
| 原类运行 | **`Inner` / `7`** | [results/original.out](results/original.out) |

即 `getSimpleName()` 从原类的 `Inner` 变为 `RF$Inner`——**可编译且行为不同**，违反 `recover-return-in-do-while-false` 与 handoff "池形类型名的结构反射陷阱" 的核心不变量。

## 为何是 ncl 引入的净倒退

- **ncl 之前**：`RF$Inner.class` 在 `decode.rs::source_internal_name` 被拒（含 `$`）→ `simpleName()` 整方法引注 → **响亮失败**（用户明确看到"这里没恢复"）。
- **ncl 之后**：decode 准入放宽 → 折叠失败回退分离呈现 → 池形 `RF$Inner.class.getSimpleName()` 发布为"已恢复"（0 引注）→ 家族可编译 → **静默偏离**。

从"诚实的未恢复"退化为"不诚实的错误恢复"，正是核心不变量要防止的。实现者的 standalone 旗标（`pool_spelled_members`）只覆盖单类口径；family 口径折叠失败时旗标为 false（build 恢复方法时折叠尚未尝试，无法预知成败），故守卫不触发。

## 与 A12-jar 的关键区别（决定修复不能靠 build 层旗标）

| fixture | 口径 | 折叠 | 最终呈现 | 结构反射 | 正确性 |
| --- | --- | --- | --- | --- | --- |
| A12-jar | family | **成功** | `Nested.class`（源码形，折叠重拼） | getSimpleName | ✓ 放行 |
| RF-jar | family | **失败**（`<clinit>`） | `RF$Inner.class`（池形，分离呈现） | getSimpleName | ✗ 须拒绝 |

二者在 build 层旗标相同（family 口径 flag=false），**区别只在 facade 折叠成败**——故守卫必须在**折叠决策之后**，build 层旗标无法区分。

## 裁决与修复方向（root）

**阻塞：ncl 不得带着此边界合入 main。** 修复方向（复用既有能力，非新机制）：facade 折叠失败回退分离呈现时（facade.rs:1957/1970 的 `Ok(Err(_reason)) => {}` 路径），对受影响方法以 `pool_spelled_members=true` **重跑恢复**——分离呈现本就是池形的，重跑会让 build 层守卫对结构反射消费形产生**真实 Refusal**（带 BCI、run 自己的解释，非伪造引注）。重跑能力有先例：facade.rs:20519 的 `analyze_method_ir`（member fold re-recovery analysis）。

约束：A12-jar（折叠成功）放行、A12-single（standalone flag=true）拒绝（已做）、N2-midLevel（getName，非结构反射）放行、RF-jar（折叠失败）拒绝（新堵）。若取证发现重跑在 facade 现有结构内确有不可逾越障碍（provenance/outcome 缝），停手报告具体障碍，由 root 决定是否收窄 ncl 准入范围。

原类为行为基准（`Inner` / `7`）。
