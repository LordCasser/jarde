## ADDED Requirements

### Requirement: Container lookup obeys the declared search position

对已显式声明的 container，类名定位 SHALL 只访问解析该位置必需的祖先链、该 container 的目录与实际候选；MUST NOT 为单位置查名展开未搜索的 sibling 或 descendant container。冷路径允许为证明候选集合完整而读取该 container 的完整中央目录。查名得到 Missing 或 Ambiguous 之前 MUST 证明对应位置的目录完整；局部成功不能声称整棵 artifact tree 完整。

#### Scenario: Unrelated nested sibling

- **WHEN** 调用方只声明一个 nested container 为查名位置，同一外层归档还含大量 sibling JAR
- **THEN** 查名读取该位置和必要祖先，不展开 sibling；新增 sibling 内容不增加 sibling 物化量，候选选择和顺序保持不变

#### Scenario: Incomplete selected directory

- **WHEN** 被搜索 container 的目录因损坏、预算或取消未完成，已读前缀没有目标名称
- **THEN** 返回未决范围及实际停止原因，不返回完整 Missing，也不转向后续 root 伪造唯一命中

#### Scenario: Unrelated damage remains outside local coverage

- **WHEN** 未搜索 sibling 已损坏，但被搜索 container 及其祖先可完整读取
- **THEN** 局部请求能完成其声明范围；显式整树请求仍报告坏 sibling，不将局部结果扩大成整树健康声明

### Requirement: Locating and reading share live container facts

单次操作的定位、绑定复核与所选 entry 读取 SHALL 消费同一组仍被该操作持有的完整权威目录和验证 backing，不得仅因阶段切换重建相同事实。此行为 MUST 不依赖跨请求 cache 已启用；操作结束释放局部持有，所选 entry 的身份及内容完整性检查仍须成立。

#### Scenario: Cross-request retention is disabled

- **WHEN** 跨请求保留关闭，当前操作已取得目标 container 的完整目录与 backing，随后读取刚定位的 class
- **THEN** 操作复用仍持有的事实，不重新枚举该目录或重复展开父容器；实际 class 读取与校验继续按剩余预算执行

#### Scenario: Local reuse cannot validate caller-supplied metadata

- **WHEN** 当前操作持有完整目录，但调用方传入的 entry metadata 与权威记录不符
- **THEN** 拒绝不匹配身份，不能因为局部事实已存在就跳过 locator/metadata 检查
