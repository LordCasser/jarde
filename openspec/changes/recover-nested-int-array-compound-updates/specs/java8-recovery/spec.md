## ADDED Requirements

### Requirement: Evidence-gated nested int array compound update

系统 SHALL 在嵌套 int 数组元素的原始行数组来源、左值复制身份、旧值读取、int 加法、写入及依赖顺序完整闭合时恢复元素 compound 更新。输出 MUST 保留每个下标/行/RHS的求值次数与顺序、异常次序和原始行对象身份，并保留相关物理指令来源；来源或消费关系不足时 MUST 沿既有拒绝契约保留原始范围，不呈现猜测更新。停止或取消 MUST 不发表局部结构或消费未闭合依赖。

#### Scenario: Captured two-dimensional int array element

- **WHEN** 已证明捕获的二维int数组在lambda helper内按准确复制后的同一行/下标读取旧值并加上Integer解箱值后写回
- **THEN** 完整helper与完整类恢复为可编译Java，原P02两真实编译器输入的输出均为 `6\n`，已有capture/SAM与其它成员保持

#### Scenario: Rank descent and scalar control

- **WHEN** 支持域内一维、二维和三维int数组元素使用同一左值身份进行加法更新
- **THEN** 输出保持各自真实数组rank、下标及最终值，一维现有更新不退化

#### Scenario: Lvalue effects and failure precede RHS

- **WHEN** 行下标、元素下标和RHS均有可观察调用，或外层/行数组为null、下标越界
- **THEN** 成功时每个调用一次且按左至右顺序；外层读取失败先于元素下标/RHS，元素读取失败先于RHS，输出异常类别、调用轨迹和原程序一致

#### Scenario: RHS replaces the outer row

- **WHEN** 旧元素读取后RHS把外层数组的对应行换成新行
- **THEN** 更新仍写入原先保存的行对象，新行值保持RHS所写内容，不重新读取外层行来选择写入目标

#### Scenario: Different lvalues or unproved source

- **WHEN** 读取与写入使用不同row/index、副本有额外消费者或原始行类型不能证明为int数组
- **THEN** 不把它们误fold成同一compound；只呈现独立证明的普通赋值或保留完整拒绝与物理来源

#### Scenario: Atomic stop and physical origins

- **WHEN** 在嵌套compound分析/构造时预算耗尽、取消或依赖闭包失败
- **THEN** 返回既有停止或拒绝状态，不发表局部结构；成功输出的复制/读取/加法/写入与行/index/RHS来源可追溯到该实际物理方法
