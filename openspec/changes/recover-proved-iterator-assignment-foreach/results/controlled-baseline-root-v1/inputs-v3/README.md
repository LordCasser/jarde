# CF11 外层赋值控制源码（v3，准备稿）

v3 基于 v2，只修正 `noCastAssign` 的源类型证据：target 改为 `Object target = Runner.seed();`；Runner 新增不写 trace、返回类型明确为 `Object` 的 `seed()`。其余 target 源码字节保持不变。该调用在 class 文件中的返回 descriptor 是 `()Ljava/lang/Object;`，保留 no-cast Assign 拒绝控制，同时避免由 String 字面量初始化导致 Jarde 把目标局部误判为 String 的歧义。没有增加类型推断机制，也没有修改产品代码。

root 提供的 v1 实测结果：baseline collector 观察到 29 条命令、58 份原始 stdout/stderr，checker exit 0；12 个原/JADX runtime case 完全一致。Jarde 输出中四个控制类编译失败，其中 `noCastAssign` 把 `Object target = "seed"; target = iterator.next();` 写成了 String target，导致 Object RHS 赋值不兼容。该问题属于独立的类型推断债务；本控制输入只通过 Object 返回 descriptor 固定初始化表达式的静态类型，不处理或混修该产品债务。

v3 源码和 manifest 尚未编译、运行或经过 JADX/Jarde，所以上述 v1 实测不代表 v3 已通过完整类验收。root 可用同一 pinned full-class collector 与其既有 launcher 模式运行新 manifest，并使用独立输出目录。准备过程没有运行构建、工具链或 Git，也没有编辑仓库。

## 观测范围

正例在循环外声明并初始化 raw `String target = "seed"`，循环首句为 `target = (String) iterator.next();`。成功赋值后 runner 记录值；正常结束时方法返回 target。异常展开后 runner 不能直接读取局部变量，只能看到成功赋值事件、失败前调用和副作用 trace，以及异常类型。

`hasNext-after-write` 仅在 `phase == hasNext && cursor > 0` 时抛出。双元素输入令第一次 hasNext、next 和赋值 observer 成功，第二次 hasNext 才抛错。拒绝 controls 保留 no-cast Assign、前置可观察调用、额外 next 消费三种边界。既有固定 CF11 诊断仅证明 AST Assign 与 next17 → CheckCast22 → Store25 路径；它没有发布 LocalVariable 身份，不能从 slot 或名称推断绑定身份。
