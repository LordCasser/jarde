# 泛型声明投影巡查（2026-10-04，root）——三形归属判定与一处未登记残留

巡查动机：5.3 里程碑 agent 施工期间，按 Goal "持续巡查各个反编译语法分析模块，构造各种 Java 语法场景" 探查 diamond/泛型声明域。用主线 HEAD 二进制（`559b9fea`）对 `-g:none` 与 `-g` 两种编译产物做单变量判别。**结论：三形中两形是既有已验收切片的已登记边界（非缺口），一形是未登记残留（低危，登记不立片）。**

固定转录见 [fixture](fixture/) 与 [results](results/)。

## 判别矩阵

| 形态 | 源码 | 主线 Jarde | 归属 |
| --- | --- | --- | --- |
| 静态泛型字段，**从不被读** | `static List<String> unread` | `List<java.lang.String>` ✓ | 已恢复 |
| 静态泛型字段，**被同类读**（`getstatic`） | `static List<String> readInMain` + `main` 读它 | **raw** `java.util.List` ✗ | **已登记边界**（见下） |
| 实例泛型字段 | `public List<String> instField` | ✓（`-g` 与 `-g:none` 均恢复） | 已恢复 |
| diamond 初始化字段 | `static List<String> initDiamond = new ArrayList<>()` | ✓ | 已恢复 |
| 方法**参数→返回** | `List<String> paramEcho(List<String> in){return in;}` | ✓ 参数与返回都带实参 | 已恢复（DT-18 首片） |
| 方法返回 `null`（**实例**、公有无参） | `public List<String> empty(){return null;}` | ✓ | 已恢复（`recover-proved-parameterized-null-return` 6/6） |
| 方法返回 `null`（**静态**） | `static List<String> nullRet(){return null;}` | **raw** ✗ | **已登记 Non-Goal**（见下） |
| 方法返回**新建泛型对象** | `List<String> newRet(){return new ArrayList<String>();}` | **raw** ✗ | **未登记残留**（见下） |

**判别方法说明**：初判疑似"静态字段泛型丢失"，但 G2/H 探针证明与 static/instance、`-g`/`-g:none`、diamond 初始化**均无关**（`Signature` 属性在 `-g:none` 下同样存在，两种口径都恢复）。真实判据是**该字段是否被同类 `Fieldref` 读取**。

## 归属一：同类读泛型字段 → `recover-same-class-generic-bindings`（11/11）的已登记残留

`readInMain` 的拒绝文本为 `field_generic_body_unproved`："a same-class Fieldref names this field and descriptor"。该片 design.md 第 24 行**明文登记**了此边界：

> "泛型字段类型改变链式方法调用的适用性 → 证明字段的所有受影响消费者；**首片不能完整追踪的链保留 `field_generic_body_unproved`，不做文本替换**。"

故这是既有已验收切片**有意保留**的保守边界，不是未实现的缺口。**不立 spec**；若要推进，应作为该片的后续（追踪字段全部消费者链），需先确认链追踪能否在既有 SSA/Program 事实内完成。

## 归属二：静态 `return null` → `recover-proved-parameterized-null-return`（6/6）的已登记 Non-Goal

该片 What Changes 明写只准入"顶级普通类的**公有无参实例** `List<String>` 返回"。`nullRet` 是 static，落在其范围外。**不立 spec**（扩大该片的准入面属独立小片，优先级低）。

## 未登记残留：泛型**新建对象**返回形（低频、低危）

`newRet(){ return new ArrayList<String>(); }` 拒绝文本：`ordinary_generic_source_unproved`："same-run Program/SSA cannot prove the body under parameterized types"（落点 `src/class_source.rs:4191`，其判据句式为 "return source is not a proven parameter value or selected member creation"）。

**严重度实测（关键）**：

- `javac --release 8` 重编 **exit 0**（可编译）；
- 运行 `n.newRet().add("x"); size()` → **`size=1`，与原类一致**（无行为差）；
- 唯一偏差：反射 `getGenericReturnType()` 由原类的 `java.util.List<java.lang.String>` 变为 **`interface java.util.List`**。

即这是**反射元数据保真度**缺口，不是行为缺口——与 DT-18 report 第 5 行已登记并接受的 `empty(){return null;}` 形**同类**（其原文："它仍能编译运行，但反射从 `java.util.List<java.lang.String>` 变为 `interface java.util.List`"）。DT-18 的那一处已由 null-return 片闭合，本形未闭合且**未在任何 change 中登记**（`grep -rln "selected member creation" openspec/changes/` 为空）。

**机制问题的分析（Goal 要求"是否一定要新增机制"）**：**不需要新机制**。`recover-proved-parameterized-null-return` 的做法是"让现有普通方法参数化声明门**消费** DT-16 的同轮候选"——本形同理，需把"返回源是**已选成员创建**（`new ArrayList<String>()`）"纳入同轮候选集，而拒绝文本本身已点名 `selected member creation` 是判据的一支（即接口已预留、只是该支未被证明覆盖）。故这是既有候选通道的**覆盖面扩展**，不是新 IR/新证明机制。

**triage 决定：登记不立片。** 理由：(1) 严重度低——可编译、行为一致，仅反射元数据降级；(2) 非静默行为错值，不触发本会话确立的核心不变量；(3) 队列已有 5 项且 5.3 大颗粒里程碑在飞，按 Goal "优先完成大颗粒语法的里程碑，以 MVP 思维推进" 不应插队；(4) 真实代码中"直接 `return new ArrayList<>()` 作为泛型返回"的频率低于"字段被读"与"参数回传"，后者均已覆盖。

若后续要推进，最小片形为：扩展同轮候选以覆盖"返回源是已选成员创建"，验收锚为本 fixture 的 `newRet` 反射恢复 `List<String>` + `paramEcho`/`empty` 零回退 + raw 负控制不变。

原 class 为行为基准（`newRet=java.util.List<java.lang.String>` / `paramEcho=java.util.List<java.lang.String>` / `size=1`）。
