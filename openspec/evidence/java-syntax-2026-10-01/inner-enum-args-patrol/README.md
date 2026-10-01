# InnerClasses family 巡查：枚举构造实参与常量专属匿名体（2026-10-01）

账本长期挂起项（DT-13 注："两层普通嵌套 enum 和接口声明子形态在固定样例上通过，但不代表 `TestInnerEnums` 的全部构造实参或 `TestEnumsInterface` 的常量专属匿名体追平"）的定向取证（主线 `edda289f`）。固定转录 [fixture](fixture/)（N0 基线 / N1=TestInnerEnums 形 / N2=TestEnumsInterface 形 / N3 判别探针，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| N0 基线：单参 String 构造 `A("a")` | 完整折叠（`A("a"), B("b");`，values/valueOf/clinit 归入 enum 呈现） |
| N3$Simple：自定义 `(byte, String)` 双参构造（纯字面量实参） | 折叠失败 → 逐字段呈现（`public static final … A;`），`valueOf` 整方法拒绝 |
| N3$Refs：自定义单参构造，实参为**兄弟枚举常量 getstatic**（`Refs(Simple.A)`） | 同上折叠失败 |
| N1$Numbers（TestInnerEnums 全量形态）：双层嵌套 + `(byte, NumString.ONE)` | 同上（10 处 @bytecode 引用） |
| N2$Operation（TestEnumsInterface 形态）：常量经**匿名子类**构造（`new Operation$1`） | 折叠失败 → 逐字段呈现（4 引用）；匿名子类 `Operation$1/$2` 独立可见 |

## 根因

折叠入口的构造 **descriptor 白名单**只有四种固定形（`(String,int,int)`/`(String,int)`/`(String,int,String)`/`(String,int,String[])` + 委托链）——任何其它用户参数形态直接 `refuse("the enum has an unsupported constructor descriptor")`。两探针（N3$Simple 纯字面量 vs N3$Refs getstatic）证明根因同源：**任意实参**是门槛，getstatic 本身不构成额外障碍。

关键架构事实：`Refs(Simple.A)` 的源码拼写就是**静态字段引用** `Simple.A`——折叠的等价性义务是"初始化等价"，实参为单条 `getstatic`（owner/name/descriptor）时按限定名拼写即忠实，**无需跨类读取兄弟枚举表**（不引入跨类证明机制）。

## 切片划分

- **`recover-proved-enum-arbitrary-arguments`（本片，MVP）**：构造 grammar 从四固定形扩展为 `(String, int, <≤3 用户实参>)`，实参语法 = {int 族字面量（按参数类型拼写窄化 `(byte) 1`）、String 字面量、静态字段引用 `Owner.name`、null}；ctor 体纪律沿用（super 调用 + 每参存 own final 字段）。闭合 N0 之外的全部纯实参形态（N3 两探针、N1 全量形态）。
- **常量专属匿名体（`N2$Operation` 形态）另片**：需要匿名子类链识别（Sub extends enum、其 <init> 委托 super、body 呈现进常量体 `PLUS { … }`），是大颗粒里程碑，随后按证据推进。**字节码结构已预研**（2026-10-01）：`<clinit>` 每常量为 `new N2$Operation$N; dup; ldc "NAME"; iconst ord; invokespecial Sub.<init>(String,int); putstatic NAME`；匿名子类 `final class Sub extends Operation`，ctor 体恰为委托 `super(name, ord, null)`（枚举自身 ctor 带合成 `$1` 参防递归），覆盖方法体（`apply`）在 Sub 类内。跨类读取（Sub 的成员表与方法体）可评估走已归档的 member-class-family 装配通道，不必然新增机制——设计时核对 A16 单类读取预算的边界。

原 class 为行为基准；JADX 参照不作为语义正例。
