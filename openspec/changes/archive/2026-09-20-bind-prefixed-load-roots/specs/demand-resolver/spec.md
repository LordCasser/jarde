## ADDED Requirements

### Requirement: Explicit entry prefixes define container load positions

系统 SHALL 允许调用方以物理 container origin 和 raw entry prefix 声明一个加载位置。prefix SHALL 为空或以 `/` 结尾；查名以 `prefix + internal_name + .class` 作逐字节精确匹配，不归一化路径或改写物理身份，不要求 ZIP 中存在显式目录 entry。不同 prefix 属于不同运行环境声明；单独发现布局节点 MUST NOT 自动激活对应位置。Standalone CLASS 的身份读取保持独立，不用文件名推断内部名。

#### Scenario: WAR application class binding

- **WHEN** 同一 container 有 `WEB-INF/classes/com/demo/A.class`，请求显式声明 `WEB-INF/classes/` 并绑定其真实物理定义
- **THEN** 对 `com/demo/A` 的选择和 driver/caller binding 可以命中该 entry，结果保持原始带前缀 raw name、ordinal、snapshot 和 origin chain

#### Scenario: Prefix is absent or malformed

- **WHEN** 调用方只声明空 prefix 或仅提供布局识别结果，或声明非空但不以 `/` 结尾的 prefix
- **THEN** 空 prefix/布局识别不能自动绑定带前缀类；不合法的 prefix 返回环境诊断，不通过补分隔符猜测用户声明

#### Scenario: Raw bytes are not normalized

- **WHEN** 两个 entry 的 raw name 在大小写、非 UTF-8 字节、反斜杠或点段上不同
- **THEN** 仅精确匹配组合名称的 entry 成为候选，不因显示转义或路径规范化合并物理定义

### Requirement: Prefixed roots preserve definition selection discipline

前缀加载位置 SHALL 遵循既有 loader 委派与 root 声明顺序、重复项歧义、缺失和不完整范围语义。选中候选的 Header 内部名 MUST 与所请求名称一致；损坏或不一致候选不得通过继续搜索后续位置隐藏。改变 prefix 后不得复用旧环境的绑定或缺失结论；共享物理目录不等于共享解析结果。

#### Scenario: Root order and duplicates

- **WHEN** 同名类同时出现在带 prefix 的应用目录及 nested library，或在同一位置重复出现
- **THEN** 跨位置遵循调用方声明顺序；同位置重复保持各自 origin 并报告 Ambiguous，不按 archive 遍历顺序或内容摘要静默消歧

#### Scenario: Header name disagrees with the entry path

- **WHEN** 前缀匹配到 `p/A.class` 但其 Header 声明另一个内部名
- **THEN** 返回该物理候选上的不一致诊断，不把该文件绑定为 A，也不绕过它选取后续 root 的 A

#### Scenario: Environment changes after an unbound result

- **WHEN** 同一物理 snapshot 先以空 prefix 请求得到 unbound，随后显式声明正确 prefix
- **THEN** 新请求重新应用其环境并可建立真实 binding，不被旧 unbound/missing 结论污染
