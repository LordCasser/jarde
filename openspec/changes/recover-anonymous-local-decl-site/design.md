# Design — `recover-anonymous-local-decl-site`

见 [proposal.md](proposal.md)。本文件把 root 2026-10-04 实测确认的事实钉成判据，使实现者不必在四处关键位置自行猜测。**本片是 `recover-anonymous-mixed-super-capture`（已验收，合并 `e1c89d57`）的下一环**，复用其捕获证明与参数角色划分，不新增第二套机制。

## Context（root 实测事实，非推理）

冻结锚 `tests/fixtures/proved-java-structure/anonymous-super-args/` 的分配点形态（root 逐行读源 + javap + 渲染实测）：

```java
public static void main(String[] args) {                      // ← 根方法是 void
    final String captured = text("capture", "captured");       // ← 捕获值来自前置局部
    Base instance = new Base(text("super-label","explicit"),   // ← 分配点在局部声明初始化位
                            number("super-value", 17)) {       //    且在方法体中部（非末尾 return）
        @Override String render() { event("body:"+captured); return super.render()+":"+captured; }
    };
    System.out.println(EVENTS);                                // ← 分配点之后还有语句
    System.out.println(instance.render());                     // ← 该局部随后被读取并调用成员
}
```

fixture 以 `javac --release 8 -g:none` 冻结（README 第 5–9 行），故 **class 内无 `LocalVariableTable`/`LineNumberTable`**（root 实测 `javap -l` 计数为 0）。当前主线呈现为物理文本：`AnonymousSuperArgs$1 local2 = new AnonymousSuperArgs$1((java.lang.String) text(...), number(...), local0);`——`$1` 非合法 Java 标识符，故完整源集 `javac` **exit 1**（响亮失败，非静默）。

## 四处 root 必须先钉死的判据（proposal 未写明，实现者不得自行选择）

### 判据 1：根方法门不能沿用"返回类型"表述——锚的根方法是 `void`

`project_class_source_anonymous_super` 现有门要求 `root_method.descriptor == "()L{parent};"`（`facade.rs` 约 4698–4703，拒绝码 `anonymous_super_return_type_unproved`）。**锚的根方法 `main` 返回 `void`，该表述对其根本不适用**。本片放宽后的判据必须是：

> 根方法为 `ACC_STATIC` 或实例方法均可，但**分配点所在语句必须是"局部变量声明且初始化值恰为该唯一已证分配点"**；父类源码名由该声明的**初始化值 `new` 操作数的 owner 类型**给出，而**不是**由根方法返回描述符给出。

即"根返回类型"这一信息来源在本形中被"**声明初始化值的 new owner**"取代。不得保留对返回描述符的任何检查（否则锚永远被拒），也不得改为"忽略返回类型"（那会放宽到未取证的范围）。

### 判据 2：分配点位于**方法体中部**，且其后有语句——站点扫描的既有放宽不覆盖它

`recover-anonymous-mixed-super-capture` 已把 `class_source_direct_return_new` 从"恰 1 条语句"放宽为"前导全为 `StmtKind::Declare` + **末条**为直返"。**锚的分配点既不是前导 Declare 也不是末条语句**（其后还有两条 `println`）。故本片必须再放宽站点扫描到"方法体内某条 `Declare` 语句的初始化值是已证唯一分配点"。

**放宽的边界（必须保持的不变量）**：
- 该方法内**已证分配点总数仍须为 1**（复用 `_anonymous_return_sites.len() == 1` 语义，改为对"声明初始化位"计数）；两个及以上仍拒绝（多分配点负例 `two-mixed-sites` 已冻结，不得回退）。
- 捕获值来源仍须可证：锚中 `captured` 来自前置 `final String captured = text(...)`，其局部槽恰一次写（`recover-anonymous-mixed-super-capture` 已实现该判据，直接复用）。
- 分配点**之后**的语句不参与投影，但**必须整体保持 `quality=structured` 且无引注**——若后续任一语句未恢复，整方法回落物理文本（不得只投影前半段，那会产出混合了投影与物理名的文本）。

### 判据 3：左端类型重拼的**健全性依据**（不是权宜之计，须按此实现）

物理文本的左端类型名 `AnonymousSuperArgs$1` 来自分配点 `new` 的 owner（无 LVT 可用）。重拼为父类源码名 `Base` 的健全性论证如下，实现者须在代码注释中保留该论证要点：

> Java 源码中**匿名类型无法被命名**，故原源码里该局部的声明类型必然是**匿名类的某个父类型**（`Base` 或 `Base` 的父类/接口）。把声明类型重拼为 `Base` 后，该局部后续任何成员访问在**静态解析**上至多比原声明更精确，不会新增原本不可解析的调用；而动态派发由 `new Base(...) { ... }` 初始化器中的匿名体保持，故**运行时行为与原 class 一致**。

**由此得到的必要检查（缺一项即可能静默偏离，必须实现）**：
- 重拼后的声明类型必须是**可在源码中拼写的名字**：父类 binary 名不含 `$`、每段是合法 Java 标识符、与根类同包（复用 `facade.rs` 约 4686–4695 的 `anonymous_super_source_type_unproved` 判据，不得另写一套）。父类为嵌套类（binary 名含 `$`）时**拒绝**——这正是 `anonymous-capture`/`anonymous-top-level` 仍被拒的原因，本片不得顺带放宽。
- **池形类型名结构反射陷阱判据适用**：若最终文本仍含 `$` 且该名字被 11 个结构反射方法之一消费（`getSimpleName`/`getEnclosingClass`/… 见 `facts.rs::STRUCTURAL_REFLECTION_METHODS`），保持拒绝。判据与 `recover-nested-class-literal-values` 一致。
- 该局部**被后续语句读取**时（锚第 31 行 `instance.render()`），重拼不得改变其可解析性：若重拼后某个读取点无法在 `Base` 上解析（例如匿名体声明了 `Base` 没有的成员且被读取），**必须回落物理文本**（响亮失败），不得发射不可编译或改变绑定的文本。

### 判据 4：proposal 的"四道前置"实为**三道**（文内不一致，以本文件为准）

proposal.md 第 3 行写"还要越过**四道**本片未钉死的前置"，但其后只列出 3 项（站点扫描、根方法门、左端重拼）。root 复核确认**实际就是 3 道**——第 4 道（"父类 binary 名不含 `$`、可拼写"）不是本片新增前置，而是 `recover-anonymous-mixed-super-capture` 已存在且本片**必须继续遵守**的既有门（判据 3 第一条）。实现者按 3 道新增 + 1 道既有约束理解，不要因为找不到"第四道"而自行发明。

### 判据 5（root 2026-10-04 追加，**架构决定性**）：站点扫描是**两路径共享**的，放宽必须显式遏制在父类路径内

root 读码确认的调用链（实现者不得假设两路径各自独立扫描）：

```
class_source_direct_return_new (report.rs:6596)          ← 已被 mixed-super-capture 放宽一次
  └─ class_source_anonymous_return_site (report.rs:533)
       └─ facade.rs:8885-8917 循环 **全部 method_asts** 累加 → `_anonymous_return_sites`
            └─ facade.rs:2091 前置 `_anonymous_return_sites.len() == 1`
                 └─ project_class_source_anonymous_interface (facade.rs:3293，解构 `[site]`)
                      └─ 3367 委派 project_class_source_anonymous_super   ← 本片目标路径
```

即：**接口匿名路径与父类匿名路径消费同一个站点向量**，接口投影在前、并在 3367 委派给父类投影。故若本片只把 `class_source_direct_return_new` 放宽到接受"局部声明初始化位"，会产生一个**未被本片取证、未被任何测试覆盖的能力激活**：一个当前产 `len()==0`（因而完全不投影）的类，放宽后可能产 `len()==1` 从而**走接口匿名投影**。这落在本片 Non-Goals 之外，属"放宽共享判据顺带打开另一条路径"的典型越界。

**必须按此实现（三选一，root 指定第 1 种）**：

1. **（root 指定）把站点形态作为一等事实携带，并在接口路径显式守住既有语义**：站点元组增加一个判别位（如 `AnonymousSiteShape::{DirectReturn, LocalDeclInitializer}`，由 `class_source_direct_return_new` 的调用方按语句位置给出）。**接口路径的前置改为要求该位为 `DirectReturn`**（即与放宽前逐字节等价），父类路径接受两位。这样遏制是**显式且可测**的，不依赖"接口路径碰巧因别的原因拒绝"。
2. 不放宽共享 helper，改为在父类路径内**另行**派生局部声明位站点。代价：两套站点派生逻辑并存，违反"不新增第二套机制"的既有约束，且 `hidden_outer_argument_bci` 等通道要接两遍——root 不推荐。
3. 放宽共享 helper 且**同时**取证接口路径的局部声明位形。代价：本片范围翻倍、接口路径的左端重拼与捕获判据都要重新取证——超出 MVP，且与 `recover-proved-anonymous-local-capture`（6/6，已验收）产生重叠。**不得走此路**。

**必须的回归测试（按第 1 种实现时）**：
- 一个**冻结负例**：类内某方法在局部声明初始化位含一个已证匿名**接口**分配点（`new I() { … }` 赋给局部），且该方法/类当前不投影。放宽后**必须仍不投影**（呈现逐字节相同）。这直接守住判据 5 的遏制。
- `recover-proved-anonymous-local-capture`（6/6）、`recover-proved-anonymous-inner-this`（8/8）、`anonymous-interface-basic`（DT-05）的全部既有测试**逐字通过**，且 corpus 双腿扫描中**接口匿名形的渲染零差异**。
- 若 corpus 双腿扫描出现**接口路径**的任何差异，即为遏制失效信号，**停下报告**，不得以"看起来正确"放行。

**顺带登记（非本片工作）**：`mixed-super-capture` 对同一 helper 的第一次放宽（"前导 `Declare` + 末条直返"）**同样**作用于接口路径。其 corpus 双腿扫描报告 49 渲染仅 1 处差异（新锚），故经验上未观察到接口路径变化；但这是**corpus 覆盖所限的阴性结果**，不是遏制证明。root 将其登记为独立债务：接口路径应补一个"前导 Declare + 末条直返"形的冻结负例，确认该放宽未激活接口投影。见 `openspec/evidence/jadx-feature-inventory-2026-09-27/`（DT-05 匿名接口域）待扩验项。

## Goals / Non-Goals

**Goals**：局部声明初始化位的混合参数匿名类内联为 `Base instance = new Base(args…) { … };`；左端类型名随投影重拼为父类源码名；锚 `anonymous-super-args` 完整源集从 `javac` exit 1 变为 exit 0 且 `java -Xverify:all` 事件日志逐行一致。

**Non-Goals（不得越界，越界即停手报告）**：
- 不放宽 `anonymous_super_return_type_unproved` 到"根方法返回父类的**子类型/接口**"（如返回 `Renderer`）——那是 `anonymous-capture`/`anonymous-top-level` 的阻塞门，**属独立未立项切片**，须先单独取证其对"分配点唯一性"与"捕获值来源可证"两条不变量的影响。
- 不支持**根方法带参数**的形（`anonymous-super-dispatch` 的 `create(String captured)`）——同属上述独立切片。
- 不支持 `this$0` + 捕获 + super 实参三者并存、嵌套匿名、多分配点、跨类引用。
- 不改 emitter 的其它呈现路径；不为局部声明形新增第二套词法替换通道。

## Decisions

1. **单一投影入口**：仍在 `project_class_source_anonymous_super` 内完成（捕获证明 + 角色划分 + 站点选择 + 左端重拼同一趟装配内闭合），不引入第二个投影函数。理由：`recover-anonymous-mixed-super-capture` 已确认该函数是原子发布接缝（其 tasks 1.1(d)），分成两处会产生半投影缝。
2. **重拼失败一律回落物理文本**，不做"部分重拼"或"用 `var`/`Object` 兜底"。理由：兜底类型会改变静态解析结果，可能把原本不可编译的文本变成可编译但绑定不同的文本——属禁止的静默偏离。
3. **复用既有拒绝码**，仅在确需区分时新增。判据 1 放宽后 `anonymous_super_return_type_unproved` 应不再由"返回类型不匹配"触发，但**仍须由"父类不可拼写"触发**（判据 3 第一条沿用 `anonymous_super_source_type_unproved`）。不得删除这两个码（corpus 双腿扫描依赖它们区分既有拒绝形）。

## Risks / Trade-offs

- **风险：放宽站点扫描到方法体中部会牵动"分配点唯一性"不变量。** 缓解：判据 2 明确要求计数语义不变，且 `two-mixed-sites` 负例已冻结、不得回退；验收须含"同方法内两个声明初始化位各含一个分配点"的负例。
- **风险：左端重拼是 emitter 新能力，可能与既有 `AnonymousOverride` 路径的隐藏实参通道（`hidden_outer_argument_bci`）交互。** 缓解：验收须含 `recover-anonymous-mixed-super-capture` 的全部正负例零回退（其新锚 `anonymous-super-mixed-direct` 是直返形，必须仍逐字节相同）。
- **风险：无 LVT 的 fixture 与有 LVT 的真实 class 可能走不同分支。** 缓解：验收须**双腿**覆盖——锚（`-g:none`，无 LVT）与一个 `-g` 编译的同形对照（有 LVT），确认两者的左端类型来源都是 `new` owner 而非 LVT，避免实现无意中依赖调试信息。

## Migration Plan

无数据迁移。落地顺序：判据 1（根方法门）→ 判据 2（站点扫描）→ 判据 3（左端重拼 + 三项检查）。三者缺一即锚不可编译，故须一次交付；但实现顺序按此推进可让每步的失败原因可定位。

## Open Questions

- 判据 2 的"分配点之后语句必须整体 structured"是否会与既有 `explanation_only` 方法（`present-proved-java-structure` 5.3 原文提到"任一方法 `explanation_only`/混合 fallback 则保持分开"）产生冲突？实现者须在取证阶段确认锚的 `main` 在放宽后仍全程 `structured`，若出现混合 fallback 则停手报告，不得自行放宽 5.3 的该项要求。
