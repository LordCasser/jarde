## ADDED Requirements

### Requirement: Immutable facts can be consumed without per-method reconstruction

同一可信读取的完整结构 SHALL 可由活动消费者共享；仍持有事实时，逐方法消费不得重新解析、重新摘要同一 backing 或完整复制其 CP/成员 payload。cache 保留权与活动请求的持有权 MUST 分开；清空、满容量或禁用跨请求保留不撤销当前操作的有效事实。共享参数和声明性配置 MUST 不成为可变的跨方法状态，返回结果仍绑定各自 origin。

#### Scenario: Cache hit hands facts to several methods

- **WHEN** 同一 class 的完整事实被多个方法消费
- **THEN** 验证计数中不出现每方法一次完整 payload 复制或同 backing 重哈希，逐方法结果拥有各自的来源和执行记录；清空 store 后活动消费继续有效（B02/B09；A15/A18）

### Requirement: Bulk retention belongs to the operation

显式批量操作 SHALL 拥有有界的事实保留作用域，直到相关消费结束；启用该操作不改变普通 Budget/单请求的默认保留策略。完整容器/backing 的容量限制、完整发布和失效规则继续适用。首版不得引入隐式进程全局 store；超容量时允许在剩余额度内直接执行，并报告重建代价。活动事实、store 保留权重、等待结果和 RSS MUST 独立报告，不能通过释放 store 引用伪造活动内存已经释放。

#### Scenario: One sweep crosses the store capacity

- **WHEN** 导出语料大于保留容量，且各类只处理一次
- **THEN** 处理完的类事实/IR 可以释放；当前消费不因准入拒绝重读，保留和在途上限不被突破，结果语义不变，后续确需重建的成本如实计费（B09；A15）

#### Scenario: Ordinary requests still have explicit retention

- **WHEN** 另一个调用方使用普通单请求入口且没有提供 store
- **THEN** 不因批量功能安装隐式共享 cache 或后台任务；批量导出清理后不保留属于它的无主事实（B09；A16/A18）
