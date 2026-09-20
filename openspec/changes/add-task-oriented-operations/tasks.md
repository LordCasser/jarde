## 1. 目标选择与请求契约

- [ ] 1.1 实现统一目标选择：接受 friendly 名称或既有物理身份并收敛到唯一物理身份，复用导航匹配与歧义规则；用两写法对照、同名多定义、重载方法与不属于本 snapshot 的身份 fixture 验证，歧义时不执行分析、身份不匹配返回输入错误（A07、A18）。
- [ ] 1.2 实现操作自选 stage 与发布：每个操作按固定 pass 表选择所需 stage 并在报告中发布实际集合；验证同一 fixture 上用显式 stage 复现相同调度，`analyze_method` 的显式列表与 `ir_pass_prerequisite_missing`/`ir_pass_order_invalid` 语义不变（A16）。
- [ ] 1.3 实现有界默认预算与少量覆盖并发布生效 `Limits`/`UsageSnapshot`：用覆盖截断工作的 fixture 验证返回真实 Partial/Cancelled 与终止维度，未知维度或非法值返回输入错误，不静默取默认（A14）。

## 2. 环境策略

- [ ] 2.1 实现单 `.class`、plain JAR 与显式 classpath 三种策略，复用既有 environment validator 并显式声明 roots/delegation/module mode；验证 standalone 无伪造 container/entry、nested 库不被自动激活、显式 root 顺序决定同名定义选择（A07、A08）。
- [ ] 2.2 验证不推断边界：用含 Manifest `Class-Path`、WAR/Boot layout 检出与嵌套库的 fixture 证明策略不生成 roots；请求未提供的 WAR 布局策略返回明确 unsupported/无效输入，不返回声称等价于容器加载的 roots（A07、A14）。

## 3. 组合操作与结果组织

- [ ] 3.1 实现类视图：一次类 Header 读取 + 一次成员列举 + 按需方法 Body，共享一个总预算并逐方法保留阶段结果/coverage/execution/诊断；用读取计数与变异（逐方法重读 Header、重复列举必须变红）验证，abstract/native 方法与损坏成员仍按 A13 隔离（A13、A16）。
- [ ] 3.2 验证 `open` 轻量与重复操作一致性：只打开时不读取 Header/Body、不构建 resolver/CFG/SSA/Region/Java AST；同一不可变 snapshot 上重复操作得到相同事实与身份（A18），该路径存在复用时启用与未启用一致（A15、A16、A17）。
- [ ] 3.3 实现按 owning method 组织的引用结果，保留常量池候选、结构引用与解析到声明三类及各自 evidence/BCI/resolution；用未使用 `Methodref`、`Sub` 调用/`Base` 声明与类级/resource 命中 fixture 验证，分组不改变 coverage/execution，未决候选不被补全（A01、A03、A11、A14）。
- [ ] 3.4 实现恢复呈现顺序：先 content、再 quality、最后停止原因，全部读取既有报告字段；用含语句、仅说明、停止三类 fixture 与“说明文本含 `return`”对照验证呈现不重新分析文本，库与 CLI 同字段（A13、A14）。

## 4. 文档与门禁

- [ ] 4.1 更新公开 API 说明与示例（目标选择、默认预算与生效配置、环境策略、类视图、引用组织、恢复呈现），明确 WAR 布局策略与跨请求复用的依赖边界和未实现范围；运行恢复、解析、查询与隔离回归（A16、A17）。
- [ ] 4.2 运行 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked`（两个固定 seed）、`cargo test --test p3_execution_comparison --locked -- --ignored`、`cargo run --example resolve_and_analyze --locked -- tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class` 与 `openspec validate --all --strict --no-interactive`，记录固定 SHA 与结果后同步/归档。
