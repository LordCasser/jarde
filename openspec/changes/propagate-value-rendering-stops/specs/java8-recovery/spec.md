## MODIFIED Requirements

### Requirement: Functional adaptation retains provenance and bounded execution

新增适配的 descriptor 解析、参数计划和表达式构造 SHALL 在共享方法预算下进行；预算或取消拒绝 SHALL 保留为停止结果，MUST NOT 经普通形状拒绝变成 fallback。已发布正文与来源 SHALL 对应真实 invokedynamic/捕获输入，证据选择 MUST NOT 改变正文。

#### Scenario: Planning budget stops at the adapter
- **WHEN** 真实函数式 site 的适配计划在新增 descriptor 或参数工作收费时触及工作上界
- **THEN** 方法 SHALL 报告相应预算维度和 site，MUST NOT 发布半适配或把停止记成 `jre_lambda_*` 形状拒绝

#### Scenario: Nested value construction propagates a stop
- **WHEN** 已选方法的值表达式递归到 lambda，新增 Local/Cast 节点的 IR 收费或取消轮询被拒绝
- **THEN** 停止 SHALL 经值渲染链到构建报告，MUST NOT 被降为局部 bytecode fallback 或普通 Java 成功

#### Scenario: Accepted adaptation keeps provenance
- **WHEN** 站点带有已证明的捕获输入并完成适配，分别请求 essential 与完整来源
- **THEN** 两者正文 SHALL 相同；完整来源 SHALL 保留 site/CP 与真实捕获 producer BCI，不能给源码 Cast 虚构 checkcast BCI

#### Scenario: Unsupported conversion remains a refusal
- **WHEN** 工厂要求当前证明边界外的参数/返回转换，且预算可用
- **THEN** 原 `jre_lambda_*` 拒绝 SHALL 保持可见，不得被错误标为资源停止或输出未经证明的 Java
