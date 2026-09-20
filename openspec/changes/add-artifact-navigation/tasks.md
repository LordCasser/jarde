## 1. 类导航列举

- [ ] 1.1 实现物理范围上的类条目列举，返回 standalone root 或 container/entry 身份并保留重复物理定义；以同名同字节多 origin fixture（上层目录入口与显式展开的嵌套库）验证条目数、origin/ordinal 与 raw name，不合并、不 first-wins（A07、A08）。
- [ ] 1.2 实现 entry 候选列举与 header 确认列举两种证据等级：候选列举零 Header 读取，确认列举真实读取 Header 并把 `this_class`/class flags 带入条目；用损坏 entry、路径名与 `this_class` 不一致、预算中断与取消 fixture 验证诊断、可靠前缀与非 Complete execution，确认未读取候选不进入确认集合（A13、A14、A16）。

## 2. 成员列举与身份绑定

- [ ] 2.1 在确认读取之上实现成员列举：方法项带 raw name、descriptor、flags 与 `PhysicalMethodId`，字段/类级/resource 命中保留自身 kind；验证列举不读取 Body、abstract/native 不伪造 Code、选择任一方法项后方法请求读取同一物理定义（A13、A16）。
- [ ] 2.2 验证列举不启动分析：只列举时 resolver/CFG/SSA/Region/Java AST 构造为 0、读取记录只有 Header；对损坏方法成员验证可靠前缀与逐成员诊断（A13、A14、A17）。

## 3. 名称查找与歧义

- [ ] 3.1 实现 friendly 名称查找：点分隔类名、内部名、成员 descriptor 与过滤条件；命中唯一时绑定真实物理身份（location/ordinal/class bytes/descriptor），无匹配时返回空候选与已扫描范围。用两种类名写法对照、重载方法与空匹配 fixture 验证，确认显示名不进入身份（A07、A13、A14）。
- [ ] 3.2 验证歧义路径：同名多定义与同名多 descriptor 返回全部候选及选择依据，不静默取第一个；回传所选物理身份后得到唯一结果，遍历顺序或路径排序的变化不改变结果身份（A07）。

## 4. 文档与门禁

- [ ] 4.1 更新库 API 说明与示例（列类、列方法、身份交接），明确列举不解析、不加载、不构建 IR，也不宣称 WAR 前缀 root 或自动加载策略；运行 reader/facade 相关测试与 A16/A17 隔离回归（A16、A17）。
- [ ] 4.2 运行 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked`（两个固定 seed）、`cargo test --test p3_execution_comparison --locked -- --ignored` 与 `openspec validate --all --strict --no-interactive`，记录固定 SHA 与结果后同步/归档。
