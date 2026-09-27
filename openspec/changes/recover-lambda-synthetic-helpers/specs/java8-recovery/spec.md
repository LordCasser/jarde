## ADDED Requirements

### Requirement: Recompilable projection of compiler-generated lambda helpers

当类源码投影能从当前物理类的完整事实证明一个 LambdaMetafactory 站点准确指向该类的私有静态 synthetic lambda helper，且 helper 的完整方法体在支持范围内可被源码投影时，系统 SHALL 将 helper 的已证明计算写入 lambda 体。系统 SHALL 仅在同一完整类范围证明该 helper 的每个引用都由已识别并成功投影的 LambdaMetafactory 实现句柄持有、且省略不会移除任何普通调用或其它成员可观察引用时，才省略 helper 声明。整组 helper 的投影与省略 MUST 原子提交；预算、取消、缺失成员/方法体、未支持指令或不完整引用清点 MUST 阻止不完整 helper 集合被省略。无法证明时 MUST 保留未投影的成员及可定位的拒绝信息。

#### Scenario: 0/1/2 参数无捕获 helper 可安全投影

- **WHEN** 无捕获 Java 8 LambdaMetafactory 站点唯一指向同类的私有静态 synthetic lambda helper，SAM 参数为 0、1 或 2 个原始 `int`，helper 完整方法体是本规则支持的直线表达式，且全类引用清点证明只由已投影站点引用
- **THEN** lambda 体直接包含 helper 的计算，类源码省略该 synthetic helper；原始、JADX 与 Jarde 完整源码均可由 `javac --release 8` 编译，且运行结果经 `java -Xverify:all` 与原 class 一致

#### Scenario: Helper has another or unknown use

- **WHEN** helper 被普通 invoke、其它未支持的动态站点、无法解析的成员引用使用，或全类引用清点未完整结束
- **THEN** 系统 MUST NOT 省略该 helper；不得根据 `lambda$` 名称或单个 bootstrap 站点推断它没有其它使用

#### Scenario: Incomplete or unsupported helper body

- **WHEN** helper 声明缺少精确私有/静态/synthetic 标志、方法体缺失或部分读取，或其控制流、异常、effect、调用、类型适配超出本片可证明的直线表达式范围
- **THEN** 系统 MUST NOT 把部分方法体内联成 lambda，也 MUST NOT 省略相关 helper；输出保留成员和拒绝来源

#### Scenario: Class-wide projection stops before commit

- **WHEN** helper 用途清点、方法读取或投影因预算/取消而未完成
- **THEN** 系统 MUST NOT 发布只省略部分 helper 的完整类投影，且报告保留实际停止原因与已读取的物理来源
