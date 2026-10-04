# 方法引用巡查（2026-10-04，root）——确认属 DT-27 已登记残留，拒绝是有原则的响亮失败

按 Goal "持续巡查各个反编译语法分析模块，构造各种 Java 语法场景，编译，然后对比源码、jadx、jarde" 巡查**方法引用六形**。主线 HEAD 二进制（`559b9fea`/`324ca011`）。

**结论：不立 spec。** 三形正确恢复；一形（绑定实例引用 `m::inst`）被拒，但拒绝理由**有原则**（改变 NPE 时机）且属 inventory **DT-27** 明文登记的残留；其余形呈现为显式 lambda 而非 `::` 语法，同为 DT-27/`recover-lambda-inline-bodies` 已登记后续。**全程为响亮失败（带引注），无静默偏离。**

固定转录见 [fixture](fixture/) 与 [results](results/)（`M1.java`/`M1.class`/`fam.jar`/`orig.out`；渲染 `M1-single.txt`、`M1-family.txt`、重编对照）。

## 探针六形与结果

| # | 形 | 源码 | 主线 Jarde | 性质 |
| --- | --- | --- | --- | --- |
| 1 | 静态方法引用 | `xs.forEach(M1::stat)` | 显式 lambda `(Object p0) -> M1.stat((String) p0)` | 行为正确（`count=1`） |
| 2 | **绑定实例引用** | `m::inst` → `f.apply("x")` | **拒绝**（4 处引注） | 响亮失败，理由有原则 |
| 3 | 任意对象实例引用 | `String::length` | 显式 lambda `(Object p0_) -> ((String) p0_).length()` | 行为正确（`5`） |
| 4 | 构造器引用 | `Item::new` | 显式 lambda `-> new M1$Item((String) p0__)` | 行为正确（`made`） |
| 5 | 数组构造器引用 | `String[]::new` | 显式 lambda + 伴生 `M1.lambda$main$1` | 行为正确（`3`） |
| 6 | Stream 链含两处引用 | `.filter(...).map(M1::stat).count()` | 显式 lambda + 伴生 | 行为正确（`count=1`） |

**原类基线**（5 行）：`x?` / `5` / `made` / `3` / `count=1`。family 口径重编 `javac --release 8` **exit 0**，运行输出 4 行（`5`/`made`/`3`/`count=1`）——**缺失的 `x?` 正是形 2 被拒的语句**，其位置在渲染文本中带 `// @bytecode` 引注标记，即 jarde **如实标注了未恢复**，未静默丢弃。

## 形 2 的拒绝理由有原则（非能力缺口）

四条引注中最关键的是 BCI 55：

> `adapting this bound receiver would move its null failure from functional-value creation to invocation`

这是**正确**的语义判断：Java 的 `m::inst` 在**创建函数值时**即对 `m` 做 null 检查（`m` 为 null 则创建处抛 NPE），而把它呈现为 `(p) -> m.inst(p)` 会把 null 失败**推迟到调用时**。二者异常时机不同，故拒绝改写而非产出"可编译但异常语义不同"的文本——与本会话确立的核心不变量一致（`recover-return-in-do-while-false` 口径）。其余三条引注是该拒绝的级联（BCI 45 "belongs to no shape this run verified"、46/49/44 "the copy at BCI 45 has no proved local assignment"、70 "reads `local3`, and no statement of this body declared that local"——因声明该局部的写入被拒，故读取处不可命名）。

## 归属（查重结果，不重复立项）

- **inventory DT-27**（"静态、实例、构造器方法引用"）已有两片验收：[非泛型隔离对照](../../java-syntax-2026-09-27/dt27-method-ref-audit/report.md) 证明 `Math::abs`、`this::number`、`RuntimeException::new` 三方一致；[泛型目标首片](../../java-syntax-2026-09-28/dt27-typed-functional/root-acceptance.md) 证明 `Function<String,Integer>` 的 `Integer::parseInt`、`this::length` 与 `Supplier<String>` 的 `this::label`。其登记原文即：**"其它泛型目标、`Object::toString` TODO、重载选择与参数/返回适配仍待扩验，不能据此清项"** —— 本巡查的形 1/3/4/5/6（呈现为显式 lambda 而非 `::`）与形 2（绑定接收者适配）正落在该残留内。
- `recover-lambda-inline-bodies`（7/7）明写：**"`::` 方法引用语法不在本片（行为已对，登记后续）"**。
- 故 `::` 源码形恢复与绑定接收者适配都是**已登记的后续工作**，不是未发现的缺口。

## 处置

- **不立 spec**（避免与 DT-27 重复立项，见 handoff "立项查重" 纪律）。
- 本巡查的价值是**新增数据点**：把"绑定实例引用被拒"的确切理由（NPE 时机）与"六形中五形行为正确、一形响亮拒绝"的实测结果记入 DT-27 的扩验证据，使将来推进 DT-27 时不必重造探针。
- 优先级判断：该残留属**呈现润色**（行为已正确、失败是响亮的），低于队列中的能力增量项（5.3 混合参数匿名类内联使 fixture 从不可编译变可编译）。

## 一处待将来核实的观察（低危，非本片结论）

形 5/6 的 lambda 伴生方法（`lambda$main$0`/`$1`/`$2`）在渲染文本中**以原名声明为成员**并被 lambda 体调用，family 重编 `javac` exit 0、行为正确。而 `recover-lambda-inline-bodies` 的文档描述复杂体走"保留但重命名为 `lambda$…$jarde` 后缀"路径——本例既未内联也未见 `$jarde` 后缀。因**可编译且行为一致**，不构成缺陷；但与文档描述的对应关系值得在推进 lambda 域时一并核实（可能是"简单体但未被内联"的第三种路径，或文档描述与实际路径的措辞差）。未在此另立项。

原 class 为行为基准（`x?`/`5`/`made`/`3`/`count=1`）。
