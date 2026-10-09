## Context

基线类ReturnedIntArrayUpdates有11个完整成员。plain2的真实Code为`aload0; iload1; aaload; iload2; dup2; iaload; iload3; iadd; dup_x2; iastore; ireturn`。dup2保存左值给旧值读；iadd之后dup_x2又保存新值，并把行引用/index与新值送给store，底部新值送给紧邻return。SSA应核对这些具体复制输出，不能把描述简写成“允许sum任意多用途”。scalar/三维/traced/replaceRow具有同一最终消费形状。

JADX实测输出保存行引用、下标和sum为局部变量，再写回和return，四完整类/Runner执行均与原程序一致。特别是replaceRow保留旧行后RHS替换外层行，返回17，旧行17、新行100；不能重新读外层行后写回。traced成功trace123，outer失败trace1，inner失败trace12。Runner额外逐一检查四个成功返回值，防止只证明写回而遗漏返回语义。

当前StmtKind::IndexAssign已有AssignOp，ExprKind::LocalAssign只写局部，ExprKind::PostfixUpdate返回旧值；Return只接受Expr。添加一个值语义的数组赋值Expr是缺失的表示能力。复制JADX临时变量路线会额外要求不存在的合成局部绑定/生命周期/身份管理，当前闭环没有必要引入它。不能据此重构所有赋值节点或顺带扩大任意赋值表达式支持。

## Goals / Non-Goals

**Goals:** 恢复一维/嵌套int数组加法更新的新值紧邻return；保持全部成员、真实类型/物理身份、左值/RHS一次求值与异常顺序；复用现有预算停止、证明提交和来源维护。

**Non-Goals:** 其它操作符/元素类型、field/local赋值表达式、非紧邻消费者、任意sum共享、row-Phi、copy全局类型传播、临时变量基础设施和全AST赋值重构。已有查找/扫描计费债务独立登记，不混进本片。

## Decisions

### 一个值语义节点，沿既有表达树走

新增最小数组赋值Expr（具体名称由实现匹配现有命名），复用现有AssignOp与array/index/value。构造点只允许证明后的int Add结果；结果类型为已证明int，assignment precedence与LocalAssign一致。完整处理emitter、类型、字段/类型名遍历和源映射等所有相关Expr匹配点。保留现有IndexAssign语句和postfix输出，不先做统一赋值框架。

### 准确额外复制，准确最终消费者

复用当前原始行int[]、dup2四副本、读值/int加法以及prefix/RHS依赖闭包。增加dup_x2时必须是category-1的row/index/sum输入，四输出互异，底部sum仅由紧邻ireturn消费，其它三个按准确顺序仅由iastore消费；之前的dup2 store副本只能供这个dup_x2，旧值与RHS只能供准确iadd，sum只能供dup_x2。iadd、dup_x2、iastore、ireturn在同块依次紧邻。方法返回类型必须int，不能仅凭opcode猜测表达式值类型。

不能一般性放松single_use_at。公共左值/RHS证明可以抽取小帮助函数，或在既有计划加入准确returned claim；不新增第二套全方法分析pass，不改不相干proof。所有新增增长循环复用Budget/Stop，固定四输出比较不做无界收集。

### Store与Return是一份所有权

借鉴已有返回postfix在最终ireturn构建表达式的做法：复制/读/add/store只有一次认领，未闭合不跳过store，结果只在Return处发布。构建或预算停止不能留下消费标记、局部Site或半个赋值文本。成功原始BCI与array/index/RHS依赖完整保留，Return自己的物理来源仍可查询。

### 完整输入和真实对照

把已运行的ReturnedIntArrayUpdates完整source、两编译器class和固定Runner/oracle移为canonical fixture，不能重写被测方法再称是同一输入。加ordinary结构/来源和ignored全类重编运行；明确升级旧NestedIntBoundaries.returned，merged仍是独立未证明row-Phi。补真实额外消费/错误复制/非int控制，复用已有对抗IR控制，不扩支持域来凑通过。

### 基于实测的局部构建磁盘约束

全仓双seed/全workspace门禁仍使用20GiB空闲停止线。定向nested/returned测试与CLI同类冷构建已实测占用约309MiB；局部命令允许在本仓target累计不超过1GiB、机器剩余不少于2GiB的双重守卫下执行。每2秒检查，只终止自己的进程组，不清其它项目，也不将其用于全workspace。明确记录每次实际阈值、target峰值和停止状态；不修改任何正确性测试或验收标准。

## Risks / Trade-offs

- 新Expr遗漏递归/命名/类型匹配会造成错误来源或不完整Java；先读全部相关walker，编译器穷尽检查和完整类对照一起验收。
- 若只检查sum共享次数，会把错误dup_x2位置/类别或错误store副本接受；必须验证有序输入输出身份与具体消费者。
- 直接重复lvalue会破坏异常顺序和RHS换行对象；原生Java compound表达式保持一次求值，十条oracle和返回值断言验证它。
- 上一片returned拒绝是范围边界，支持域变化后应精准改其预期；不得减弱仍未支持merged或原source-map条件。

## Migration Plan

无兼容层。当前main上一片确切CI验收之前，Luna仅产出可审阅patch，不修改冻结产品源；root验收前片后再apply、测试和建立新CLI身份。新片必须追踪自身产品commit的完整CI，不能借前片绿灯。

## Open Questions

若同一次实际证据发现既有完整表达设施已能表达新值写回，则优先复用并删去拟议节点；必须给出完整输出、物理消费者和求值证明，不能将临时变量理论可行当作现有机制已可用。
